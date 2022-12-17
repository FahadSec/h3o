use super::h3api;
use float_eq::assert_float_eq;
use h3o::{CellIndex, Resolution};

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let result = index.area_m2();
            let reference = h3api::cell_area_m2(index);

            assert_float_eq!(
                result,
                reference,
                r2nd <= f64::from(f32::EPSILON)
            );
        }
    };
}

test!(hexagon_res0, 0x8009fffffffffff);
test!(hexagon_res1, 0x81083ffffffffff);
test!(hexagon_res2, 0x820807fffffffff);
test!(hexagon_res3, 0x830800fffffffff);
test!(hexagon_res4, 0x8408001ffffffff);
test!(hexagon_res5, 0x85080003fffffff);
test!(hexagon_res6, 0x860800007ffffff);
test!(hexagon_res7, 0x870800000ffffff);
test!(hexagon_res8, 0x8808000001fffff);
test!(hexagon_res9, 0x89080000003ffff);
test!(hexagon_res10, 0x8a0800000007fff);
test!(hexagon_res11, 0x8b0800000000fff);
test!(hexagon_res12, 0x8c08000000001ff);
test!(hexagon_res13, 0x8d080000000003f);
test!(hexagon_res14, 0x8e0800000000007);
test!(hexagon_res15, 0x8f0800000000000);

test!(pentagon_res0, 0x8073fffffffffff);
test!(pentagon_res1, 0x81737ffffffffff);
test!(pentagon_res2, 0x82734ffffffffff);
test!(pentagon_res3, 0x83734efffffffff);
test!(pentagon_res4, 0x84734a9ffffffff);
test!(pentagon_res5, 0x85734e67fffffff);
test!(pentagon_res6, 0x86734e64fffffff);
test!(pentagon_res7, 0x87734e64dffffff);
test!(pentagon_res8, 0x88734e6499fffff);
test!(pentagon_res9, 0x89734e64993ffff);
test!(pentagon_res10, 0x8a734e64992ffff);
test!(pentagon_res11, 0x8b734e649929fff);
test!(pentagon_res12, 0x8c734e649929dff);
test!(pentagon_res13, 0x8d734e64992d6ff);
test!(pentagon_res14, 0x8e734e64992d6df);
test!(pentagon_res15, 0x8f734e64992d6d8);

// Apply a cell area calculation function to every cell on the earth at a given
// resolution, and check that it sums up the total earth area.
macro_rules! area_earth_test {
    ($name:ident, $resolution:literal, $tolerance:literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let area = CellIndex::base_cells()
                .flat_map(|index| {
                    index.children(resolution).map(|child| child.area_m2())
                })
                .sum::<f64>();
            let expected = 510065621724088.6; // Earth surface, in m²

            assert_float_eq!(area, expected, abs <= $tolerance);
        }
    };
}

area_earth_test!(earth_at_res0, 0, 1e0);
area_earth_test!(earth_at_res1, 1, 1e5);
area_earth_test!(earth_at_res2, 2, 1e0);
area_earth_test!(earth_at_res3, 3, 1e3);
area_earth_test!(earth_at_res4, 4, 1e2);
