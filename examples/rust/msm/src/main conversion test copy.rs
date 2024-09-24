use lambdaworks_math::elliptic_curve::short_weierstrass::curves::bls12_381::default_types::FrElement;
use lambdaworks_math::unsigned_integer::element::UnsignedInteger;

fn main() {
    // 16진수 문자열을 FrElement로 변환
    let hex_values = [
        "0x0a8adbd8f210ae643e474fff49d535ef3cfb1561539d0156176aa81860047441fd0b2810fe8ad4d53762c37cbecc1bb3",
        "0x05f2f9bd70a6055c4358b905992296cb24e5de8df9df3599d029b7cc53b1575de16d95d7ab673106f8fd420a7e9f46ea",
    ];
    let fr_elements: Vec<UnsignedInteger<6>> = hex_values
        .iter()
        .map(|&hex| 
            UnsignedInteger::<6>::from_hex(hex).unwrap()
        )
        .collect();

    // let hex_values_back: Vec<String> = fr_elements
    //     .iter()
    //     .map(|fe| {
    //         // 각 FieldElement의 UnsignedInteger에 접근
    //         let value = fe.value(); // UnsignedInteger 값을 가져옴
    //         value
    //             .limbs
    //             .iter()
    //             .map(|limb| format!("{:016x}", limb)) // 각 limb을 16진수로 변환
    //             .collect::<String>()
    //     })
    //     .collect();

    println!("FrElement: {:?}", fr_elements);
    // println!("Reverted hex values: {:?}", hex_values_back);
}

