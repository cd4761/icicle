use lambdaworks_math::{
    cyclic_group::IsGroup,
    elliptic_curve::traits::IsPairing,
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{curve::BLS12381Curve, default_types::FrElement},
            point::ShortWeierstrassProjectivePoint,
        },
        traits::IsEllipticCurve,
    },
    errors::DeserializationError,
    field::{element::FieldElement, traits::IsPrimeField},
    msm::pippenger::msm,
    traits::{AsBytes, Deserializable},
    unsigned_integer::element::UnsignedInteger,
};

use std::str::FromStr;
type G1Point = <BLS12381Curve as IsEllipticCurve>::PointRepresentation;

fn main() {
    let upper_points: Vec<G1Point> = vec![
        ("0x13895c2470264b895cfc3028b9252ab1f651f5ed8620358531ab2a9b97180f2a", "0x2929605273765bdfdfad9d79504cf509b2279a8d618278a2a709f6020a621d7e"),
        ("0x2a2d05eaa5567156661364a3c859136e51c443513d6b27d8f853afab2f3fa231", "0x2243ab3312d3ae63bed3c9a308b8f6f07e2a2633d8897ec75f4f3bf860e43399"),
        ("0x1d86697b11da5994b546959effb726fa3a73ccf1b008416e9b524c430a78a264", "0x23b079b80a959f9381a5e9dac827701bf2d7fb7f384507b7c6286b6b2efdb3fa"),
        ("0x25f509bfb973401a2d05805e89db628df59c43f321c758a261d369c7a9935213", "0x2e69056859a7f2dba6689352118cb4cdda5a085e3966a66bbd0eb51902a9b0ca"),
        ("0x0f325f5c30b3d4a0fc7663351d18e15bb72ffb4b65d2f24aab1a33d67e81f303", "0x16d2125661589fd67e34bcd1fdc536541ef8acd44000313d34fee4f26dcdd18f"),
        ("0x1ff3e6d0227f57a0b7b8efb7c8c71feaf6028f1254e49e086a6d89e089ab2aeb", "0x041c5f35d2920436e713e033d46de6231555fe2e6064232aa2dbda2e274b411a"),
        ("0x0e261d85b2de7df2d7006f910a9f302fdd402064179a2c8bac8f4b7a058ffcc9", "0x2d593ea7d2d66aa4d638f11839502b903ac380ecd76fbe7f0c504987941dc084"),
        ("0x07cf0fb9944ed5960836a195b16178197cd81a363193788c302e00d4e0d37cf0", "0x165d03a90012f0c281991e0e246eeffc8c040a8bd2c8efb463a49db887278595"),
        ("0x29cc9b43b9f229bbe69ce87cfa3c18e73d29b5caf3306ae1220647bf3182355e", "0x2c67ccde83f7d85d74c2847258383600da2cbc7fe37653aaf77c7e803c8d4cde"),
        ("0x1007866b82012e59986030e9e915983d3f54e7c190bcb8cbcc685e0ca616fba6", "0x182fe16ca72feaba722ea001e88fbeb972994d30952a34160bf76827caa4d32a"),
    ].into_iter().map(|(x, y)| {
        G1Point::new([
            FieldElement::from_hex(x).unwrap(),
            FieldElement::from_hex(y).unwrap(),
            FieldElement::one(),  
    }).collect();

    let upper_scalars: Vec<UnsignedInteger<4>> = vec![
        "0x1f1a54774a5468f420c0a6d5f894d68c73c5c419242ba9a5669af1e86f50da56",
        "0x0b844b4f984c5abd4c9906bdadd1cd13c47cf840c5f865228ad0ba3590baa49d",
        "0x297f73ded73d3032002be52310077a13b5536e731e8447dbb515d85bdb5cc2d8",
        "0x295c080ca228aad10a20f9ca4aa243f5c5bf3177464fcc79be4395223f6506a6",
        "0x02a5ae5a6fd720fe1d8d844f269953ce195b3de9098c6b0a28527a96131bf8ed",
        "0x00c3bd1e85c54cc5c92ac12e9aaa074d9eceff4b21cbdfd826631d6469ed8b7e",
        "0x1f987b6ad1516137c122f1089c2a5781f7a9595381d18b859259c2f2c5c47102",
        "0x2c784f247ce515d9acf5988f0aa98ae07d7648374f47c7148c8c6b0c06a25da3",
        "0x1621087f8b665210e664c8f593f6368b706d0bc75524f03ad56fc39159c2ed5d",
        "0x03f21536aeab360d7c25a7b1cfd1895f454559715d69008c02bfaa02f42efee7"
        
    ].into_iter().map(|s| UnsignedInteger::<4>::from_hex(s).unwrap()).collect();

    // MSM 수행
    let msm_result = msm(&upper_scalars, &upper_points).expect("MSM calculation failed");

    // 결과 출력
    println!("MSM result: {:?}", msm_result);
}
