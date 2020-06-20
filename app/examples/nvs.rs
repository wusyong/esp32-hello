#![no_main]

use esp_idf_hal::nvs::*;

#[no_mangle]
fn app_main() {
  let mut nvs_partition = NonVolatileStorage::default();

  let mut nvs = nvs_partition.open("test").unwrap();

  macro_rules! test_set_get {
    ($nvs:expr, $ty:ty, $value:expr) => {
      let original_value = <$ty>::from($value);
      let original_value_clone = original_value.clone();

      let owned_type = stringify!($ty);
      let ref_type = concat!("&", stringify!($ty));

      nvs.set(ref_type, &original_value).expect(&format!("failed setting {}", ref_type));
      nvs.set(owned_type, original_value).expect(&format!("failed setting {}", owned_type));

      let ref_value = nvs.get::<$ty>(ref_type).expect(&format!("failed getting {}", ref_type));
      let owned_value = nvs.get::<$ty>(owned_type).expect(&format!("failed getting {}", owned_type));

      assert_eq!(owned_value, ref_value, "did not get the same value for {} and {}", owned_type, ref_type);
      assert_eq!(owned_value, original_value_clone, "did not get the original value for {}", owned_type);
      assert_eq!(ref_value, original_value_clone, "did not get the original value for {}", ref_type);

      println!("Success: {} == {}", owned_type, ref_type);
    }
  }

  test_set_get!(nvs, bool, true);
  test_set_get!(nvs, i8, -8i8);
  test_set_get!(nvs, i16, -16i16);
  test_set_get!(nvs, i32, -32i32);
  test_set_get!(nvs, i64, -64i64);
  test_set_get!(nvs, u8, 8u8);
  test_set_get!(nvs, u16, 16u16);
  test_set_get!(nvs, u32, 32u32);
  test_set_get!(nvs, u64, 64u64);
  test_set_get!(nvs, String, "String");
  test_set_get!(nvs, Vec<u8>, vec![1, 2, 3, 4]);
}
