//! Stack for the TON Center v2 wire contract.

use super::Int64Input;
use serde::{Deserialize, Serialize};

/// A standard stack entry containing a serialized slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmStackEntrySlice {
    /// Fixed `TONLib` discriminator `tvm.stackEntrySlice`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmStackEntrySlice,
    /// The TVM slice value.
    pub slice: TvmSlice,
}

/// A standard stack entry containing a serialized cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmStackEntryCell {
    /// Fixed `TONLib` discriminator `tvm.stackEntryCell`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmStackEntryCell,
    /// The TVM cell value.
    pub cell: TvmCell,
}

/// A standard stack entry containing a signed TVM integer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmStackEntryNumber {
    /// Fixed `TONLib` discriminator `tvm.stackEntryNumber`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmStackEntryNumber,
    /// The TVM number value (arbitrary-precision decimal).
    pub number: TvmNumberDecimal,
}

/// A standard stack entry containing an ordered tuple.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmStackEntryTuple {
    /// Fixed `TONLib` discriminator `tvm.stackEntryTuple`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmStackEntryTuple,
    /// The TVM tuple value.
    pub tuple: TvmTuple,
}

/// A standard stack entry containing an ordered list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmStackEntryList {
    /// Fixed `TONLib` discriminator `tvm.stackEntryList`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmStackEntryList,
    /// The TVM list value.
    pub list: TvmList,
}

/// A stack value without a supported `TONLib` JSON representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmStackEntryUnsupported {
    /// Fixed `TONLib` discriminator `tvm.stackEntryUnsupported`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmStackEntryUnsupported,
}

/// A TVM slice serialized as a base64 `BoC`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmSlice {
    /// Fixed `TONLib` discriminator `tvm.slice`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmSlice,
    /// TVM slice data serialized as base64.
    pub bytes: String,
}

/// A TVM cell serialized as a base64 `BoC`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmCell {
    /// Fixed `TONLib` discriminator `tvm.cell`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmCell,
    /// TVM cell data serialized as base64.
    pub bytes: String,
}

/// A signed TVM integer represented as decimal text, without a 64-bit limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmNumberDecimal {
    /// Fixed `TONLib` discriminator `tvm.numberDecimal`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmNumberDecimal,
    /// Integer value as a decimal string (supports arbitrarily large numbers).
    pub number: String,
}

/// An ordered collection of standard `TONLib` stack entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmTuple {
    /// Fixed `TONLib` discriminator `tvm.tuple`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmTuple,
    /// Ordered tuple of TVM stack entries. Each element is a typed object with `@type`
    /// discriminator (`tvm.stackEntryNumber`, `tvm.stackEntryCell`, `tvm.stackEntrySlice`,
    /// `tvm.stackEntryTuple`, or `tvm.stackEntryList`).
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub elements: Vec<TvmStackEntry>,
}

/// A `TONLib` list containing standard stack entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TvmList {
    /// Fixed `TONLib` discriminator `tvm.list`; other values are rejected.
    #[serde(rename = "@type")]
    pub type_tag: super::tags::TvmList,
    /// Ordered list of TVM stack entries. Each element is a typed object with `@type`
    /// discriminator (`tvm.stackEntryNumber`, `tvm.stackEntryCell`, `tvm.stackEntrySlice`,
    /// `tvm.stackEntryTuple`, or `tvm.stackEntryList`).
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub elements: Vec<TvmStackEntry>,
}

/// Expanded cell data with bit length, child references, and an exotic-cell flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LegacyTvmCell {
    /// Cell binary data.
    pub data: LegacyTvmCellData,
    /// Array of child cell references, forming a directed acyclic graph (DAG). Each child is a TVM
    /// cell object with its own data (base64), refs, and special flag.
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub refs: Vec<LegacyTvmCell>,
    /// Returns `true` if this is a special (exotic) cell type such as a Merkle proof or library
    /// reference; otherwise `false`.
    pub special: bool,
}

/// Legacy cell payload containing its base64 `BoC` and an optional expanded representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LegacyStackEntryCell {
    /// Raw cell bytes in base64.
    pub bytes: String,
    /// Parsed cell object with data, refs, and type info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<LegacyTvmCell>,
}

/// Cell binary data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LegacyTvmCellData {
    /// Cell data in base64.
    pub b64: String,
    /// Data length in bits.
    pub len: i32,
}

/// Standard `TONLib` stack entry. Tuples and lists recursively retain their entry
/// tags; cell bytes remain base64 `BoCs` for the application's chosen cell library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum TvmStackEntry {
    /// A slice serialized by `TONLib` as a `BoC`.
    Slice(TvmStackEntrySlice),
    /// A cell serialized by `TONLib` as a `BoC`.
    Cell(TvmStackEntryCell),
    /// A signed TVM integer encoded as decimal text.
    Number(TvmStackEntryNumber),
    /// An ordered tuple, including the empty tuple.
    Tuple(TvmStackEntryTuple),
    /// An ordered `TONLib` list.
    List(TvmStackEntryList),
    /// A value `TONLib` cannot represent in its public JSON stack vocabulary.
    Unsupported(TvmStackEntryUnsupported),
}

impl TvmStackEntry {
    /// Creates a decimal integer stack entry without narrowing its value.
    #[must_use]
    pub fn number(value: impl ToString) -> Self {
        Self::Number(TvmStackEntryNumber {
            type_tag: Default::default(),
            number: TvmNumberDecimal {
                type_tag: Default::default(),
                number: value.to_string(),
            },
        })
    }

    /// Creates a cell stack entry from a base64-encoded `BoC`.
    #[must_use]
    pub fn cell(value: impl Into<String>) -> Self {
        Self::Cell(TvmStackEntryCell {
            type_tag: Default::default(),
            cell: TvmCell {
                type_tag: Default::default(),
                bytes: value.into(),
            },
        })
    }

    /// Creates a slice stack entry from a base64-encoded `BoC`.
    #[must_use]
    pub fn slice(value: impl Into<String>) -> Self {
        Self::Slice(TvmStackEntrySlice {
            type_tag: Default::default(),
            slice: TvmSlice {
                type_tag: Default::default(),
                bytes: value.into(),
            },
        })
    }

    /// Creates an ordered tuple of standard stack entries.
    #[must_use]
    pub fn tuple(value: Vec<Self>) -> Self {
        Self::Tuple(TvmStackEntryTuple {
            type_tag: Default::default(),
            tuple: TvmTuple {
                type_tag: Default::default(),
                elements: value,
            },
        })
    }

    /// Creates an ordered list of standard stack entries.
    #[must_use]
    pub fn list(value: Vec<Self>) -> Self {
        Self::List(TvmStackEntryList {
            type_tag: Default::default(),
            list: TvmList {
                type_tag: Default::default(),
                elements: value,
            },
        })
    }
}

/// A legacy stack entry contains exactly two array elements: its tag and payload.
///
/// Numbers in replies use hexadecimal strings; requests also accept decimal text
/// and signed 64-bit JSON integers. Unsupported entries are output-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LegacyStackEntry {
    /// Integer input or hexadecimal integer output.
    Number((LegacyNumberTag, Int64Input)),
    /// Structured cell output or structured cell input.
    Cell((LegacyCellTag, LegacyStackEntryCell)),
    /// Structured slice input; the C++ response converter emits slices as cells.
    Slice((LegacySliceTag, LegacyStackEntryCell)),
    /// Base64 `BoC` input with the `tvm.Cell` spelling.
    CellBytes((LegacyCellBytesTag, String)),
    /// Base64 `BoC` input with the `tvm.Slice` spelling.
    SliceBytes((LegacySliceBytesTag, String)),
    /// Tuple whose children use the standard `TONLib` stack representation.
    Tuple((LegacyTupleTag, TvmTuple)),
    /// List whose children use the standard `TONLib` stack representation.
    List((LegacyListTag, TvmList)),
    /// Output-only marker with an empty string payload.
    Unsupported((LegacyUnsupportedTag, String)),
}

/// Accepted integer tags for legacy stack requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LegacyNumberTag {
    /// Wire spelling `num`.
    #[serde(rename = "num")]
    Num,
    /// Wire spelling `int`.
    #[serde(rename = "int")]
    Int,
    /// Wire spelling `number`.
    #[serde(rename = "number")]
    Number,
    /// Wire spelling `integer`.
    #[serde(rename = "integer")]
    Integer,
}

/// Legacy tag for a cell object with a bytes property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LegacyCellTag {
    /// Wire spelling `cell`.
    #[serde(rename = "cell")]
    Cell,
}

/// Legacy tag for a slice object with a bytes property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LegacySliceTag {
    /// Wire spelling `slice`.
    #[serde(rename = "slice")]
    Slice,
}

/// Legacy tag for a bare base64 cell `BoC` request argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LegacyCellBytesTag {
    /// Wire spelling `tvm.Cell`.
    #[serde(rename = "tvm.Cell")]
    Cell,
}

/// Legacy tag for a bare base64 slice `BoC` request argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LegacySliceBytesTag {
    /// Wire spelling `tvm.Slice`.
    #[serde(rename = "tvm.Slice")]
    Slice,
}

/// Accepted tuple tags; replies use the lowercase spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LegacyTupleTag {
    /// Wire spelling `tuple`.
    #[serde(rename = "tuple")]
    Tuple,
    /// Wire spelling `tvm.Tuple`.
    #[serde(rename = "tvm.Tuple")]
    TvmTuple,
}

/// Accepted list tags; replies use the lowercase spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LegacyListTag {
    /// Wire spelling `list`.
    #[serde(rename = "list")]
    List,
    /// Wire spelling `tvm.List`.
    #[serde(rename = "tvm.List")]
    TvmList,
}

/// Output-only tag for unsupported TVM values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LegacyUnsupportedTag {
    /// Wire spelling `unsupported`.
    #[serde(rename = "unsupported")]
    Unsupported,
}

#[cfg(feature = "openapi")]
impl utoipa::PartialSchema for LegacyStackEntry {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        use utoipa::openapi::schema::OneOfBuilder;

        OneOfBuilder::new()
            .description(Some("Legacy stack entry: exactly [type, value]. The type determines the payload encoding; nested tuple/list entries use standard TONLib objects."))
            .item(pair_schema::<LegacyNumberTag, Int64Input>())
            .item(pair_schema::<LegacyCellTag, LegacyStackEntryCell>())
            .item(pair_schema::<LegacySliceTag, LegacyStackEntryCell>())
            .item(pair_schema::<LegacyCellBytesTag, String>())
            .item(pair_schema::<LegacySliceBytesTag, String>())
            .item(pair_schema::<LegacyTupleTag, TvmTuple>())
            .item(pair_schema::<LegacyListTag, TvmList>())
            .item(pair_schema::<LegacyUnsupportedTag, String>())
            .into()
    }
}

#[cfg(feature = "openapi")]
impl utoipa::ToSchema for LegacyStackEntry {
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        use utoipa::PartialSchema;

        schemas.push((Self::name().into_owned(), Self::schema()));
        TvmStackEntry::schemas(schemas);
        schemas.push((TvmStackEntry::name().into_owned(), TvmStackEntry::schema()));
        LegacyStackEntryCell::schemas(schemas);
        LegacyTvmCell::schemas(schemas);
        schemas.push((LegacyTvmCell::name().into_owned(), LegacyTvmCell::schema()));
    }
}

#[cfg(feature = "openapi")]
fn pair_schema<A: utoipa::PartialSchema, B: utoipa::PartialSchema>()
-> utoipa::openapi::schema::Array {
    use utoipa::openapi::{
        RefOr,
        schema::{AllOfBuilder, ArrayBuilder, ArrayItems, Schema},
    };

    let items = [A::schema(), B::schema()].map(|item| match item {
        RefOr::T(schema) => schema,
        RefOr::Ref(reference) => Schema::AllOf(AllOfBuilder::new().item(reference).build()),
    });
    ArrayBuilder::new()
        .prefix_items(items)
        .items(ArrayItems::False)
        .min_items(Some(2))
        .max_items(Some(2))
        .build()
}
