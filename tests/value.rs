#[macro_use]
extern crate serde_derive;

#[cfg(feature = "std")]
mod std_tests {

    use std::collections::BTreeMap;

    use multi_cbor::value::Value;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct TupleStruct(String, i32, u64);

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct UnitStruct;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    #[allow(clippy::struct_field_names)] // test fixture field named after its tuple-struct type
    struct Struct<'a> {
        tuple_struct: TupleStruct,
        tuple: (String, f32, f64),
        map: BTreeMap<String, String>,
        bytes: &'a [u8],
        array: Vec<String>,
        unit_array: Vec<UnitStruct>,
    }

    fn as_object(value: &Value) -> &BTreeMap<Value, Value> {
        let Value::Map(ref v) = value else {
            panic!();
        };
        v
    }

    #[test]
    fn serde() {
        let tuple_struct = TupleStruct("test".to_string(), -60, 3000);

        let tuple = ("hello".to_string(), -50.004_097, -12.094_635_556_478);

        let map: BTreeMap<String, String> = [
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
            ("key3".to_string(), "value3".to_string()),
            ("key4".to_string(), "value4".to_string()),
        ]
        .into_iter()
        .collect();

        let bytes = b"test byte string";

        let array = vec![format!("one"), format!("two"), format!("three")];
        let unit_array = vec![UnitStruct, UnitStruct, UnitStruct];

        let data = Struct {
            tuple_struct,
            tuple,
            map,
            bytes,
            array,
            unit_array,
        };

        let value = multi_cbor::value::to_value(data.clone()).unwrap();
        println!("{value:?}");

        let data_ser = multi_cbor::to_vec(&value).unwrap();
        let data_de_value: Value = multi_cbor::from_slice(&data_ser).unwrap();

        for ((k1, v1), (k2, v2)) in as_object(&value)
            .iter()
            .zip(as_object(&data_de_value).iter())
        {
            assert_eq!(k1, k2);
            assert_eq!(v1, v2);
        }

        assert_eq!(value, data_de_value);
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct SmallStruct {
        spam: u32,
        eggs: u32,
    }

    #[test]
    fn small_struct() {
        // Test whether the packed format works.
        // Field names should not be serialized,
        // instead field indizes are serialized.
        let value = SmallStruct { spam: 17, eggs: 42 };
        let data = multi_cbor::ser::to_vec_packed(&value).unwrap();
        let reference = b"\xa2\x00\x11\x01\x18\x2a";
        assert_eq!(data, reference);
    }
}
