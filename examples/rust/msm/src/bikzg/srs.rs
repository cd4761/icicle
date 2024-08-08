use icicle_cuda_runtime::{
    memory::{DeviceSlice, DeviceVec},
    stream::CudaStream,
};

use icicle_core::{curve::Curve, msm::{msm, MSMConfig}, traits::GenerateRandom};

use lambdaworks_math::{
    cyclic_group::IsGroup,
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{
                curve::BLS12381Curve,
                default_types::{FrConfig, FrElement, FrField},
                pairing::BLS12381AtePairing,
                twist::BLS12381TwistCurve,
            },
            point::ShortWeierstrassProjectivePoint,
        },
        traits::{IsEllipticCurve, IsPairing},
    },
    field::element::FieldElement,
    polynomial::Polynomial,
    traits::{AsBytes, Deserializable},
    unsigned_integer::element::U256,
};

use crate::bipolynomial::BivariatePolynomial;
use icicle_core::polynomials::UnivariatePolynomial;

pub struct StructuredReferenceString<G1, G2> {
    dimension_x: usize,
    dimension_y: usize,
    powers_main_group: Vec<G1>,
    powers_secondary_group: [G2; 3],
}

impl<G1, G2> StructuredReferenceString<G1, G2> {
    pub fn new(
        dimension_x: usize,
        dimension_y: usize,
        powers_main_group: &[G1],
        powers_secondary_group: &[G2; 3],
    ) -> Self {
        Self {
            dimension_x,
            dimension_y,
            powers_main_group: powers_main_group.to_vec(),
            powers_secondary_group: powers_secondary_group.clone(),
        }
    }
}

pub trait CommitmentScheme {
    type Commitment;
    fn commit_bivariate(&self, bp: &BivariatePolynomial<FieldElement<FrElement>>) -> Self::Commitment;
    fn commit_univariate(&self, p: &dyn UnivariatePolynomial<Field = FrField, FieldConfig = FrConfig, Element = FieldElement<FrElement>>) -> Self::Commitment;
    fn open(
        &self,
        x: &FieldElement<FrElement>,
        y: &FieldElement<FrElement>,
        evaluation: &FieldElement<FrElement>,
        p: &BivariatePolynomial<FieldElement<FrElement>>,
    ) -> (Self::Commitment, Self::Commitment);
    fn verify(
        &self,
        x: &FieldElement<FrElement>,
        y: &FieldElement<FrElement>,
        evaluation: &FieldElement<FrElement>,
        p_commitment: &Self::Commitment,
        proofs: &(Self::Commitment, Self::Commitment),
    ) -> bool;
}

pub struct KZG<P: IsPairing<ScalarField = FrElement>> {
    srs: StructuredReferenceString<
        <P as IsPairing>::G1Point,
        <P as IsPairing>::G2Point,
    >,
}

impl<P: IsPairing<ScalarField = FrElement>> CommitmentScheme for KZG<P> {
    type Commitment = <P as IsPairing>::G1Point;

    fn commit_bivariate(&self, bp: &BivariatePolynomial<FieldElement<FrElement>>) -> Self::Commitment {
        let coefficients_x_y: Vec<_> = bp
            .coefficients
            .iter()
            .flat_map(|row| row.iter().map(|&coeff| coeff.representative()))
            .collect();
        let flattened_points: Vec<_> = self.srs.powers_main_group.iter().cloned().collect();

        let stream = CudaStream::create().unwrap();
        let device_coefficients = DeviceVec::from_slice(&coefficients_x_y, &stream).unwrap();
        let device_points = DeviceVec::from_slice(&flattened_points, &stream).unwrap();

        let mut results = DeviceVec::cuda_malloc(coefficients_x_y.len()).unwrap();

        let msm_config = MSMConfig::default();

        msm(
            device_coefficients,
            device_points,
            &mut results,
            &msm_config
        ).expect("`points` is sliced by `cs`'s length");

        results.to_vec(&stream).unwrap().remove(0)
    }

    fn commit_univariate(&self, p: &dyn UnivariatePolynomial<Field = FrField, FieldConfig = FrConfig, Element = FieldElement<FrElement>>) -> Self::Commitment {
        let coefficients_y: Vec<_> = p
            .coefficients()
            .iter()
            .map(|coefficient| coefficient.representative())
            .collect();
        let first_col_powers_main_group: Vec<_> = self.srs.powers_main_group.iter().step_by(self.srs.dimension_x).cloned().collect();

        let stream = CudaStream::create().unwrap();
        let device_coefficients = DeviceVec::from_slice(&coefficients_y, &stream).unwrap();
        let device_points = DeviceVec::from_slice(&first_col_powers_main_group[..coefficients_y.len()], &stream).unwrap();

        let mut results = DeviceVec::cuda_malloc(coefficients_y.len()).unwrap();

        let msm_config = MSMConfig::default();

        msm(
            device_coefficients,
            device_points,
            &mut results,
            &msm_config
        ).expect("`points` is sliced by `cs`'s length");

        results.to_vec(&stream).unwrap().remove(0)
    }

    fn open(
        &self,
        x: &FieldElement<FrElement>,
        y: &FieldElement<FrElement>,
        evaluation: &FieldElement<FrElement>,
        p: &BivariatePolynomial<FieldElement<FrElement>>,
    ) -> (Self::Commitment, Self::Commitment) {
        let poly_to_commit = p.sub_by_field_element(evaluation);
        let (q_xy, q_y) = poly_to_commit.ruffini_division(x, y);

        let q_xy_commitment = self.commit_bivariate(&q_xy);
        let q_y_commitment = self.commit_univariate(&q_y);

        (q_xy_commitment, q_y_commitment)
    }

    fn verify(
        &self,
        x: &FieldElement<FrElement>,
        y: &FieldElement<FrElement>,
        evaluation: &FieldElement<FrElement>,
        p_commitment: &Self::Commitment,
        proofs: &(Self::Commitment, Self::Commitment),
    ) -> bool {
        let g1 = &self.srs.powers_main_group[0];
        let g2 = &self.srs.powers_secondary_group[0];
        let tau_g2 = &self.srs.powers_secondary_group[1];
        let tetha_g2 = &self.srs.powers_secondary_group[2];

        let e = P::compute_batch(&[
            (
                &p_commitment.operate_with(&(g1.operate_with_self(evaluation.representative())).neg()),
                g2,
            ),
            (
                &proofs.0.neg(),
                &(tau_g2.operate_with(&(g2.operate_with_self(x.representative())).neg())),
            ),  
            (
                &proofs.1.neg(),
                &(tetha_g2.operate_with(&(g2.operate_with_self(y.representative())).neg())),
            ),
        ]);
        e == Ok(FieldElement::one())
    }
}

pub fn g1_points_srs(dims: (usize, usize), taus: (FrElement, FrElement)) -> Vec<Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>>> {
    let powers_of_tau_theta = vandemonde_challenge(&taus.0, &taus.1, dims.0, dims.1);

    let g1: ShortWeierstrassProjectivePoint<BLS12381Curve> = <BLS12381Curve as IsEllipticCurve>::generator();
    let mut two_dim_tau_g1: Vec<Vec<ShortWeierstrassProjectivePoint<BLS12381Curve>>> = Vec::with_capacity(dims.0);
   
    for i in 0..dims.0 {
        let y_row = powers_of_tau_theta[i].clone();
        let mut tau_g1 = vec![g1.clone(); dims.1];
        tau_g1
            .par_iter_mut()
            .zip(&y_row)
            .for_each(|(g1, tau_i)| {
                *g1 = g1.operate_with_self(tau_i.representative());
            });
        two_dim_tau_g1.push(tau_g1);
    }

    two_dim_tau_g1
}

fn vandemonde_challenge(tau: &FrElement, theta: &FrElement, row_len: usize, col_len: usize) -> Vec<Vec<FrElement>> {
    let mut vec: Vec<Vec<FrElement>> = Vec::with_capacity(row_len);

    for _ in 0..row_len {
        vec.push(Vec::with_capacity(col_len));
    }

    for i in 0..row_len {
        let y_row = theta.pow(i);

        let mut row: Vec<FrElement> = Vec::with_capacity(col_len);
        for j in 0..col_len {
            row.push(y_row.clone().mul(tau.pow(j)));
        }
        vec[i] = row;
    }
    vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn create_srs() {
        let powers_main_group = g1_points_srs((2, 3), (FrElement::from(2), FrElement::from(3)))
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();

        let g2_powers = (0..3)
            .into_par_iter()
            .map(|i| {
                <BLS12381AtePairing as IsPairing>::G2Point::generator()
                    .operate_with_self(FrElement::from(2).pow(i).representative())
            })
            .collect::<Vec<_>>();

        let mut powers_secondary_group = [g2_powers[0].clone(), g2_powers[1].clone(), g2_powers[2].clone()];

        let srs = StructuredReferenceString::new(2, 3, &powers_main_group, &powers_secondary_group);

        assert_eq!(srs.powers_main_group.len(), 6);
        assert_eq!(srs.powers_secondary_group.len(), 3);
    }

    #[test]
    fn zkg_1() {
        let srs = create_srs();

        let bikzg = KZG { srs };

        let coefficients = [
            [
                FrElement::from(1),
                FrElement::from(2),
                FrElement::from(3),
            ],
            [
                FrElement::from(4),
                FrElement::from(5),
                FrElement::from(6),
            ],
        ];
        let bp = BivariatePolynomial::new(&coefficients);
        let p_commitment = bikzg.commit_bivariate(&bp);

        let x = FieldElement::from(1);
        let y = FieldElement::from(1);
        let fake_evaluation = FieldElement::from(2);

        let proof = bikzg.open(&x, &y, &fake_evaluation, &bp);

        assert!(bikzg.verify(&x, &y, &fake_evaluation, &p_commitment, &proof));
    }

    #[test]
    fn test_vandemonde_challenge() {
        let challenge = vandemonde_challenge(&FrElement::from(2), &FrElement::from(3), 2, 3);

        assert_eq!(
            challenge,
            vec![
                vec![FrElement::from(1), FrElement::from(2), FrElement::from(4)],
                vec![FrElement::from(3), FrElement::from(6), FrElement::from(12)]
            ]
        );
    }
}
