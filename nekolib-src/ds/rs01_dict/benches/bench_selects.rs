use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use rs01_dict::Rs01Dict;

use crate::benchmarks::select_word;

mod benchmarks {
    pub fn select_word<const X: bool>(mut w: u64, mut i: u32) -> u32 {
        if !X {
            w = !w;
        }
        let mut res = 0;
        for lg2 in (0..6).rev() {
            let len = 1 << lg2;
            let mask = !(!0 << len);
            let count = (w & mask).count_ones();
            if count <= i {
                w >>= len;
                i -= count;
                res += len;
            }
        }
        res
    }
}

fn bench_selects(c: &mut Criterion) {
    let w = 0x_3046_2FB7_58C1_EDA9_u64;
    let a: Vec<_> = (0..64).map(|i| w >> i & 1 != 0).collect();

    let rs = Rs01Dict::new(&a);

    let mut group = c.benchmark_group("select");

    for i in 0..32 {
        let actual = rs.select1(i) as u32;
        let expected = select_word::<true>(w, i as _);
        assert_eq!(expected, actual);
    }

    group
        .bench_function(BenchmarkId::new("mofr", w), |b| {
            b.iter(|| {
                for i in 0..32 {
                    black_box(rs.select1(i));
                }
            })
        })
        .bench_function(BenchmarkId::new("word", w), |b| {
            b.iter(|| {
                for i in 0..32 {
                    black_box(select_word::<true>(w, i));
                }
            })
        });

    // % bc <<< "obase=16; ibase=2; $(gshuf -re {0,1}{0,1}{0,1}{0,1} -n$((20*16)))" \
    //       | tr -d \\n \
    //       | fold -w4 \
    //       | paste -sd _ - \
    //       | fold -w20 \
    //       | sed 's/^/0x_/; s/_$/,/'

    let a: [u64; 128] = [
        0x_C6BF_9486_5479_7948,
        0x_79C1_A360_2AE7_EB35,
        0x_B943_93B6_E26A_A55C,
        0x_8E51_8DCB_1667_E799,
        0x_5919_2338_6C2B_86A1,
        0x_02B4_20BD_58E7_EB2D,
        0x_52F6_17E4_3FA6_C57B,
        0x_93BF_785F_A5DA_982B,
        0x_1163_7DAF_B12E_D991,
        0x_F525_DF50_28C8_7179,
        0x_8590_2989_A233_4065,
        0x_93FC_73A3_BDB4_E38B,
        0x_AA75_942E_4A64_E6CF,
        0x_168D_DE24_22D3_5FAC,
        0x_4A3A_2BB6_D6F3_FB68,
        0x_824C_DF8F_B3BF_0496,
        0x_FA7A_CEB1_F710_019C,
        0x_2117_443C_7657_11AD,
        0x_7227_4BC7_0E83_5289,
        0x_DAEF_91B3_7C70_FA6A,
        0x_F067_D525_E450_F8E6,
        0x_1718_9490_FD63_7019,
        0x_2AED_2F24_AFDA_ECE8,
        0x_5D96_4552_0877_EF66,
        0x_07FE_E469_14C8_A532,
        0x_4C4D_6BDF_29D5_7FF5,
        0x_B9DE_F058_DEFB_A2B9,
        0x_3EBA_A22D_940F_8CDC,
        0x_4537_F6BD_9665_028B,
        0x_9ED7_D1B6_AE25_9019,
        0x_9E4D_4C65_7AE1_B65C,
        0x_FD77_B006_AEF4_AF81,
        0x_5322_631F_165D_40E2,
        0x_739C_5087_F316_1567,
        0x_3F85_C218_5720_2CE1,
        0x_ACDC_E77B_2D10_4D44,
        0x_B817_9967_9F93_7C62,
        0x_4514_E648_D21D_9AE2,
        0x_E839_BB6B_F05F_23C1,
        0x_EDD0_F550_4607_CF1D,
        0x_E505_70FE_F643_8BF0,
        0x_1D81_F625_2C42_744F,
        0x_E1DD_D9AE_9B3C_88B5,
        0x_6BBA_4252_6CEE_63D2,
        0x_8E29_939D_0CC1_8057,
        0x_8CC8_5B41_C879_5995,
        0x_DBB3_CF8B_05ED_87ED,
        0x_FD8A_9E72_C198_C078,
        0x_E02B_F746_48B3_9D9F,
        0x_5879_64D8_BC41_D476,
        0x_95C0_8003_05A5_6982,
        0x_B9D2_BF1E_475B_B090,
        0x_650A_FCF0_F0AE_001A,
        0x_8896_7682_115B_655C,
        0x_0388_1B31_A1B7_762A,
        0x_1B8E_4140_05CE_E41E,
        0x_4257_12A8_4962_855C,
        0x_9DB2_268F_3A88_0BAF,
        0x_7A43_BB2B_0D44_8AA4,
        0x_4086_0FF6_812A_4B66,
        0x_3FFD_A1A9_FECC_E6E2,
        0x_95DB_0C36_5E7C_F448,
        0x_BB01_F760_DFBC_D712,
        0x_5395_D6E8_0A07_1EA8,
        0x_7F71_DD9D_0136_5623,
        0x_716A_3B05_17FA_F4D7,
        0x_BFF2_1024_F07C_9BE1,
        0x_2DAE_441D_8296_B96B,
        0x_D4CD_842E_E69C_76E9,
        0x_CB43_A7DE_BAA1_1A57,
        0x_011B_9859_D5C0_BC42,
        0x_4448_F010_3B89_A2C5,
        0x_6D57_6015_38F4_38B3,
        0x_B4D7_19B1_181D_FB88,
        0x_99CB_1CE6_7597_1C79,
        0x_353B_7AE5_FBCF_9498,
        0x_98AD_DDC1_2287_5B20,
        0x_82EF_6428_A2CC_1529,
        0x_976D_E48C_99E7_77DB,
        0x_1CEC_428C_5852_26BF,
        0x_1686_275A_DF4B_2632,
        0x_E329_6477_121A_2BC1,
        0x_5087_8F7D_18EC_2A73,
        0x_8CA7_E6AD_425E_F073,
        0x_388B_3DA5_6F94_F40B,
        0x_A598_409D_725A_3187,
        0x_8605_948C_6986_5742,
        0x_2CDB_C903_AE8B_BF4E,
        0x_5C9B_8086_46DE_6135,
        0x_F69A_12B3_40C2_7B3D,
        0x_F127_BD43_C8D7_987E,
        0x_B8CE_C2EC_2275_26AB,
        0x_769D_92E1_2636_9089,
        0x_2601_A595_1514_06F2,
        0x_2F23_F12E_07DF_5F87,
        0x_7D5F_0A44_6612_9571,
        0x_2DEB_71FF_FFBB_5396,
        0x_558E_658B_61DC_0DDA,
        0x_830C_E661_F4A2_ECD5,
        0x_F38B_263A_1B79_4582,
        0x_CAD5_71FE_1619_5FFA,
        0x_7BBE_CDBC_155E_086D,
        0x_565F_C474_32E2_DF47,
        0x_AB8E_E459_D893_7E51,
        0x_1D35_399F_67F1_46D2,
        0x_5259_B110_1D5C_0C5F,
        0x_E375_6199_C9A4_3E9E,
        0x_9F71_493E_24B5_3317,
        0x_3280_C6AF_C8A9_078D,
        0x_040B_4662_2EF0_7041,
        0x_40B1_17E1_819B_F2D8,
        0x_33AC_459E_A4D2_F0E9,
        0x_693C_7128_B52A_CCBB,
        0x_EECA_450C_45CB_B34D,
        0x_6300_1761_0062_78EE,
        0x_AAF6_37E9_094E_D1BA,
        0x_B965_EA87_D30A_E1C7,
        0x_64B6_08E9_F22B_96D5,
        0x_3725_7221_BE3C_25B0,
        0x_94ED_B82B_A857_3166,
        0x_F277_9F65_806B_AE8A,
        0x_B1F3_4BCA_D8C8_A436,
        0x_7B5B_6F7C_CDF8_66BB,
        0x_24C4_D58E_BB06_A004,
        0x_A012_F50A_9E4F_52DA,
        0x_7DD8_B04B_CD47_F9B9,
        0x_307E_E73B_B4E9_846A,
        0x_14E3_ADC7_3BEB_0889,
    ];
    let a: Vec<_> =
        a.iter().flat_map(|&w| (0..64).map(move |i| w >> i & 1 != 0)).collect();

    let rs = Rs01Dict::new(&a);

    let expected1: Vec<_> = (0..a.len()).filter(|&i| a[i]).collect();
    let actual1: Vec<_> = (0..expected1.len()).map(|i| rs.select1(i)).collect();
    assert_eq!(actual1, expected1);

    let count1 = expected1.len();
    eprintln!("count1: {count1}");

    let expected0: Vec<_> = (0..a.len()).filter(|&i| !a[i]).collect();
    let actual0: Vec<_> = (0..expected0.len()).map(|i| rs.select0(i)).collect();
    assert_eq!(actual0, expected0);

    let count0 = expected0.len();
    eprintln!("count0: {count0}");

    group
        .bench_function(BenchmarkId::new("mofr", 1), |b| {
            b.iter(|| {
                for i in 0..count1 {
                    black_box(rs.select1(i));
                }
            })
        })
        .bench_function(BenchmarkId::new("mofr", 0), |b| {
            b.iter(|| {
                for i in 0..count0 {
                    black_box(rs.select0(i));
                }
            })
        })
        .bench_function(BenchmarkId::new("array", 1), |b| {
            b.iter(|| {
                for i in 0..count1 {
                    black_box(expected1[i]);
                }
            })
        })
        .bench_function(BenchmarkId::new("array", 0), |b| {
            b.iter(|| {
                for i in 0..count0 {
                    black_box(expected0[i]);
                }
            })
        });
}

criterion_group!(benches, bench_selects);
criterion_main!(benches);
