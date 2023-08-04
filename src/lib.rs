// TODO(version: v1.0.0): license/author header project-wide, see MIT guidelines
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// TODO(version: v1.0.0): remove these and remove all unused code
#![allow(unused)]

extern crate nalgebra as na;
use std::ops::Index;

pub use num_complex;

use num_complex::Complex;
use num_traits::{One, Zero};

macro_rules! complex {
    ($re:expr, $im:expr) => {
        $crate::num_complex::Complex::new($re, $im)
    };
}

mod scalar;
pub use scalar::Scalar;

// mod roots;
// pub use roots::Roots;

mod complex_util;
use complex_util::{c_neg, complex_sort_mut};
mod impl_num;
mod num_util;

mod linalg_util;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Poly<T: Scalar>(na::DVector<Complex<T>>);

impl<T: Scalar> Poly<T> {
    pub fn new(coeffs: &[Complex<T>]) -> Self {
        Self(na::DVector::from_row_slice(coeffs))
    }

    /// ```
    /// use rust_poly::Poly;
    /// use num_complex::Complex;
    /// use num_traits::{Zero, One};
    ///
    /// let p = Poly::from_roots(&[Complex::new(-1.0, 0.0), Complex::zero(), Complex::one()]);
    /// assert_eq!(p, Poly::new(&[Complex::zero(), Complex::new(-1.0, 0.0), Complex::zero(), Complex::one()]))
    /// ```
    #[must_use]
    pub fn from_roots(roots: &[Complex<T>]) -> Self {
        if roots.is_empty() {
            return Self::one();
        }

        let mut roots: na::DVector<Complex<T>> = na::DVector::from_column_slice(roots);
        complex_sort_mut(&mut roots);

        roots
            .as_slice()
            .iter()
            .map(|e| Self::line(c_neg(e.clone()), Complex::<T>::one()))
            .fold(Self::one(), |acc, x| acc * x)
    }

    /// ```
    /// use rust_poly::Poly;
    /// use num_complex::Complex;
    /// use num_traits::{One, Zero};
    ///
    /// assert_eq!(Poly::line(Complex::one(), Complex::new(-1.0, 0.0)).eval_point(Complex::one()), Complex::zero());
    /// ```
    pub fn line(offset: Complex<T>, slope: Complex<T>) -> Self {
        if slope.is_zero() {
            return Self::new(&[offset]);
        }
        Self::new(&[offset, slope])
    }

    /// ```
    /// use rust_poly::Poly;
    /// use num_complex::Complex;
    /// use num_traits::{One, Zero};
    ///
    /// let p1 = (Complex::new(-1.0, 0.0), Complex::new(2.0, 0.0));
    /// let p2 = (Complex::new(2.0, 0.0), Complex::new(-1.0, 0.0));
    ///
    /// assert_eq!(Poly::line_from_points(p1, p2).eval_point(Complex::one()), Complex::zero());
    /// ```
    pub fn line_from_points(p1: (Complex<T>, Complex<T>), p2: (Complex<T>, Complex<T>)) -> Self {
        let slope = (p2.1 - p1.1.clone()) / (p2.0 - p1.0.clone());
        let offset = p1.1 - slope.clone() * p1.0;
        Self::line(offset, slope)
    }

    fn len_raw(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.normalize().len_raw()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_normalized(&self) -> bool {
        let n = self.len_raw();
        !self.0.index(n - 1).is_zero()
    }

    fn normalize(&self) -> Self {
        if self.len_raw() == 0 {
            return self.clone();
        }
        // while self.0.iter().last().unwrap().is_zero() {
        //     self.0.remove_row(self.len_raw() - 1);
        // }
        let mut end = self.len_raw();
        loop {
            if !self.0.iter().last().unwrap().is_zero() {
                break;
            }
            end -= 1;
        }
        Self(na::DVector::from_column_slice(&self.0.as_slice()[0..end]))
    }

    /// ```
    /// use rust_poly::Poly;
    /// use num_complex::Complex;
    ///
    /// let p = Poly::new(&[Complex::new(1.0, 0.0), Complex::new(2.0, 0.0), Complex::new(3.0, 0.0)]);
    /// let x = Complex::new(1.0, 0.0);
    /// assert_eq!(p.eval_point(x), Complex::new(6.0, 0.0));
    /// ```
    pub fn eval_point(&self, x: Complex<T>) -> Complex<T> {
        self.eval(na::DMatrix::<_>::from_row_slice(1, 1, &[x]))[0].clone()
    }

    #[must_use]
    pub fn eval(&self, x: na::DMatrix<Complex<T>>) -> na::DMatrix<Complex<T>> {
        let mut c0: na::DMatrix<_> = na::DMatrix::<_>::from_element(
            x.nrows(),
            x.ncols(),
            self.0[self.len_raw() - 1].clone(),
        );
        for i in 2..=self.len_raw() {
            c0 *= x.clone();
            c0.apply(|c| *c = (*c).clone() + &self.0[self.len_raw() - i]);
        }
        c0
    }

    #[must_use]
    pub fn pow(&self, pow: u32) -> Self {
        self.pow_usize(pow as usize)
    }

    #[must_use]
    pub fn pow_usize(&self, pow: usize) -> Self {
        // invariant: poly is normalized
        debug_assert!(self.is_normalized());

        if pow == 0 {
            return Self::one();
        }

        if pow == 1 {
            return self.clone();
        }

        // TODO: divide and conquer with powers of 2
        let mut res = self.clone();
        for _ in 2..=pow {
            res = res * self;
        }
        res
    }

    /// ```
    /// use rust_poly::Poly;
    /// use rust_poly::num_complex::Complex;
    ///
    /// let p = Poly::new(&[Complex::new(1.0, 0.0), Complex::new(2.0, 0.0), Complex::new(3.0, 0.0), Complex::new(0.0, -1.5)]);
    /// ```
    fn companion(&self) -> na::DMatrix<Complex<T>> {
        // invariant: poly is normalized
        debug_assert!(self.is_normalized());

        // pre-condition: poly has degree 1 or more
        assert!(
            self.len_raw() >= 2,
            "polynomials of degree 0 or less do not have a companion matrix"
        );

        if self.len_raw() == 2 {
            return na::DMatrix::from_row_slice(
                1,
                1,
                &[c_neg(self.0[0].clone()) / self.0[1].clone()],
            );
        }

        let n = self.len_raw() - 1;
        let mut mat: na::DMatrix<Complex<T>> = na::DMatrix::<Complex<T>>::zeros(n, n);

        // fill sub-diagonal with 1
        mat.view_mut((1, 0), (n - 1, n - 1))
            .fill_diagonal(Complex::<T>::one());

        // fill the rightmost column with the coefficients of the associated
        // monic polynomial
        let monic = self
            .0
            .view((0, 0), (n, 1))
            .map(|x| c_neg(x) / self.0[n].clone());
        for i in 0..n {
            mat.column_mut(n - 1)[i] = monic[i].clone();
        }
        mat
    }

    /// ```
    /// use rust_poly::Poly;
    /// use rust_poly::num_complex::Complex;
    ///
    /// let p = Poly::new(&[Complex::new(1.0, 0.0), Complex::new(2.0, 0.0), Complex::new(3.0, 0.0), Complex::new(4.0, 0.0)]);
    /// dbg!(p.roots());
    /// assert!(false);
    /// ```
    #[must_use]
    pub fn roots(&self) -> Option<na::DVector<Complex<T>>> {
        // invariant: polynomial is normalized
        debug_assert!(self.is_normalized());

        if self.len_raw() < 2 {
            return Some(na::dvector![]);
        }

        if self.len_raw() == 2 {
            return Some(na::dvector![c_neg(self.0[0].clone()) / self.0[1].clone()]);
        }

        // rotated companion matrix reduces error
        let mut comp = self.companion();
        let n = comp.shape().0;
        for i in 0..n / 2 {
            comp.swap_rows(i, n - i - 1);
            comp.swap_columns(i, n - i - 1);
        }

        let mut r: na::DVector<Complex<T>> = comp.eigenvalues()?;
        complex_sort_mut(&mut r);
        Some(r)
    }

    /// Compose two polynomials, returning a new polynomial.
    ///
    /// Substitute the given polynomial `x` into `self` and expand the
    /// result into a new polynomial.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_poly::Poly;
    /// use num_complex::Complex;
    /// use num_traits::identities::One;
    ///
    /// let f = Poly::new(&[Complex::new(1.0, 0.0), Complex::new(2.0, 0.0)]);
    /// let g = Poly::one();
    ///
    /// assert_eq!(f.compose(g), f);
    #[must_use]
    pub fn compose(&self, x: Self) -> Self {
        // invariant: polynomials are normalized
        debug_assert!(self.is_normalized());
        debug_assert!(x.is_normalized());

        // TODO begin: are these checks actually making things faster?
        if self.is_zero() || x.is_zero() {
            return Self::zero();
        }

        if self.is_one() {
            return x;
        }

        if x.is_one() {
            return self.clone();
        }
        // end

        // TODO: prove that composing two normalized polynomials always results
        //       in a normalized polynomial or else disprove and call .normalize()
        (0..self.len_raw())
            .map(|i| Self::new(&[self.0[i].clone()]) * x.pow_usize(i))
            .sum()
    }

    /// ```
    /// use rust_poly::Poly;
    /// use num_complex::Complex;
    /// use num_traits::identities::One;
    ///
    /// let c1 = Poly::new(&[Complex::new(1.0, 0.0), Complex::new(2.0, 0.0), Complex::new(3.0, 0.0)]);
    /// let c2 = Poly::new(&[Complex::new(3.0, 0.0), Complex::new(2.0, 0.0), Complex::new(1.0, 0.0)]);
    /// let expected1 = (Poly::new(&[Complex::new(3.0, 0.0)]), Poly::new(&[Complex::new(-8.0, 0.0), Complex::new(-4.0, 0.0)]));
    /// assert_eq!(c1.clone().div_rem(&c2), expected1);
    /// ```
    #[must_use]
    pub fn div_rem(self, rhs: &Self) -> (Self, Self) {
        // invariant: polynomials are normalized
        debug_assert!(self.is_normalized());
        debug_assert!(rhs.is_normalized());

        // pre-condition: don't divide by zero
        assert!(!rhs.is_zero(), "Attempted to divide a polynomial by zero");

        let lhs_len = self.len_raw();
        let rhs_len = self.len_raw();
        if lhs_len < rhs_len {
            return (Self::zero(), self);
        }
        if rhs_len == 1 {
            return (
                Self(self.0 / rhs.0[rhs.len_raw() - 1].clone()),
                Self::zero(),
            );
        }
        let len_delta = lhs_len - rhs_len;
        let scale = rhs.0[rhs.len_raw() - 1].clone();
        let rhs: na::DVector<_> = rhs
            .0
            .view_range(0..rhs.len_raw() - 1, 0..1)
            // HACK: this shouldn't be necessary, but nalgebra turns DVector into
            //       DMatrix when making a view, and needs to be politely reminded
            //       that this is a column vector.
            .column(0)
            .into();
        // TODO: useless clone of scale, it should be borrowed, but dvector does
        //       not implement Div<&_>
        let rhs: na::DVector<_> = rhs / scale.clone();
        let mut lhs: na::DVector<_> = self.0.clone();
        let mut i = len_delta as isize;
        let mut j = (lhs_len - 1) as isize;
        while i >= 0 {
            lhs.view_range_mut(i as usize..j as usize, 0..1)
                .iter_mut()
                .zip((rhs.clone() * self.0[j as usize].clone()).iter())
                .for_each(|p| *p.0 -= p.1);
            i -= 1;
            j -= 1;
        }
        (
            Self(
                (lhs.view_range(j as usize + 1..lhs.len(), 0..1) / scale)
                    .column(0)
                    .into(),
            ),
            Self(lhs.view_range(..(j + 1) as usize, 0..1).column(0).into()),
        )
    }
}

impl<T: Scalar> Index<usize> for Poly<T> {
    type Output = Complex<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
