use lambdaworks_math::elliptic_curve::short_weierstrass::curves::bls12_381::default_types::FrElement;
use lambdaworks_math::unsigned_integer::element::UnsignedInteger;

fn main() {
    // 16진수 문자열을 FrElement로 변환
    let hex_values = [
        "0x04c2724ca3ad45b689a038995abb0f8469662cb81e83999aac3e99c16f0888e8d5157f16be9159ab06cf7fbdf4dd7043",
        "0x11f1b6696b79700d5d763e77fae760e81525cd58f07ff163cb471c8f162d48d5bc6f46b5bea660ea33c57f053dd465b3",
        "0x0cde1fe466305ff7bbd278f287ec3cd80cc918b7dfa0a94156dd1364a3e3fe46261321728e8ec57134cbd0e236ae7686",
    ];
    let fr_elements: Vec<UnsignedInteger<6>> = hex_values
        .iter()
        .map(|&hex| {
            // 16진수 문자열을 UnsignedInteger로 변환
            UnsignedInteger::<6>::from_hex(hex).unwrap()
            // UnsignedInteger를 FrElement로 변환
            
        })
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

