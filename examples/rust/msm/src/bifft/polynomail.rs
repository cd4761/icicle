use lambdaworks_math::fft::errors::FFTError;

use lambdaworks_math::field::traits::{IsField, IsSubFieldOf};
use lambdaworks_math::{
    field::{
        element::FieldElement,
        traits::{IsFFTField, RootsConfig},
    },
    polynomial::Polynomial,
};
use alloc::{vec, vec::Vec};

#[cfg(feature = "cuda")]
use lambdaworks_math::fft::gpu::cuda::polynomial::{evaluate_fft_cuda, interpolate_fft_cuda};
#[cfg(feature = "metal")]
use lambdaworks_math::fft::gpu::metal::polynomial::{evaluate_fft_metal, interpolate_fft_metal};

use lambdaworks_math::cpu::{ops, roots_of_unity};

