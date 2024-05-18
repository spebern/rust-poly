use na::Complex;
use num::{One, Zero};

use crate::{Poly, Scalar, __util::complex::c_neg};

impl<T: Scalar> Poly<T> {
    /// The length of the polynomial without checking pre-conditions
    pub(crate) fn len_raw(&self) -> usize {
        self.0.len()
    }

    /// The degree of the polynomial without checking pre-conditions
    pub(crate) fn degree_raw(&self) -> i32 {
        self.len_raw() as i32 - 1
    }

    pub(crate) fn is_normalized(&self) -> bool {
        let n = self.len_raw();
        if n == 0 {
            // FIXME: maybe zero-polynomials should be illegal?
            return true;
        }
        // a constant is always normalized, as it may be just a constant zero
        if n == 1 {
            return true;
        }
        !self.0.index(n - 1).is_zero()
    }

    pub(crate) fn normalize(self) -> Self {
        if self.is_normalized() {
            return self;
        }
        let mut end = self.len_raw();
        loop {
            if end == 0 {
                return Self::zero();
            }
            if !self.0.as_slice()[end - 1].is_zero() {
                break;
            }
            end -= 1;
        }
        let ret = Self(na::DVector::from_column_slice(&self.0.as_slice()[0..end]));

        // post-condition: polynomial is now normalized
        debug_assert!(ret.is_normalized());
        ret
    }

    pub(crate) fn companion(&self) -> na::DMatrix<Complex<T>> {
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
        let mut monic = self.clone();
        monic.make_monic();
        for i in 0..n {
            mat.column_mut(n - 1)[i] = monic[i].clone();
        }
        mat
    }

    /// The last coefficient
    pub(crate) fn last(&self) -> Complex<T> {
        self.0[self.len_raw() - 1].clone()
    }

    /// Make the polynomial monic in-place.
    ///
    /// Monic polynomials are scaled such that the last coefficient is 1, and
    /// the roots are preserved
    pub(crate) fn make_monic(&mut self) {
        let last_coeff = self.last();
        self.0.apply(|x| *x = x.clone() / last_coeff.clone());
    }
}

#[cfg(test)]
mod test {
    use na::DVector;
    use num::{complex::Complex64, Zero};

    use crate::Poly;

    /// This was a bug
    #[test]
    fn normalize0() {
        let p = Poly(DVector::from_column_slice(&[Complex64::zero()]));
        assert_eq!(p.normalize().0.as_slice(), &[Complex64::zero()]);
    }

    /// This was a bug
    #[test]
    fn is_normalized0() {
        let p = Poly(DVector::from_column_slice(&[Complex64::zero()]));
        assert!(p.is_normalized());
    }
}