// Licensed under the Apache-2.0 license.s

use std::ops::{Neg, Not};

use mcu_registers_systemrdl_new::{
    ast::{
        AccessType, AddressingType, InstanceOrPropRef, InterruptType, OnReadType, OnWriteType,
        PrecedenceType,
    },
    Bits,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    U64(u64),
    Bool(bool),
    Bits(Bits),
    String(String),
    EnumReference(String, String),
    InstanceOrPropRef(InstanceOrPropRef),
    PrecedenceType(PrecedenceType),
    AccessType(AccessType),
    OnReadType(OnReadType),
    OnWriteType(OnWriteType),
    AddressingType(AddressingType),
    InterruptType(InterruptType),
}

impl Value {
    pub fn property_type(&self) -> PropertyType {
        match self {
            Value::U64(_) => PropertyType::U64,
            Value::Bool(_) => PropertyType::Boolean,
            Value::Bits(_) => PropertyType::Bits,
            Value::String(_) => PropertyType::String,
            Value::EnumReference(_, _) => PropertyType::EnumReference,
            Value::InstanceOrPropRef(_) => PropertyType::InstanceOrPropRef,
            Value::PrecedenceType(_) => PropertyType::PrecedenceType,
            Value::AccessType(_) => PropertyType::AccessType,
            Value::OnReadType(_) => PropertyType::OnReadType,
            Value::OnWriteType(_) => PropertyType::OnWriteType,
            Value::AddressingType(_) => PropertyType::AddressingType,
            Value::InterruptType(_) => PropertyType::FieldInterrupt,
        }
    }

    pub fn is_integral(&self) -> bool {
        match self {
            Value::U64(_) => true,
            Value::Bool(_) => true,
            Value::Bits(_) => true,
            _ => false,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Value::U64(v) => *v == 0,
            Value::Bool(v) => *v == false,
            Value::Bits(v) => v.val() == 0,
            _ => false,
        }
    }

    pub fn logical_not(&self) -> Value {
        if self.is_zero() {
            Value::Bool(true)
        } else {
            Value::Bool(false)
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Value::U64(v) => *v <= 1,
            Value::Bits(b) => b.w() == 1,
            Value::Bool(_) => true,
            _ => false,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(v) => *v,
            Value::U64(v) => *v != 0,
            Value::Bits(b) => b.val() != 0,
            _ => false,
        }
    }

    pub(crate) fn try_andand(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_oror(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_lt(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_gt(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_lte(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_gte(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_eq(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_neq(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_rshift(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_lshift(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_and(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_or(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_xor(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_xnor(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_times(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_divide(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_modulus(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_add(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_sub(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }

    pub(crate) fn try_pow(&self, rhs: &Value) -> Result<Value, anyhow::Error> {
        todo!()
    }
}

impl Not for Value {
    type Output = Value;
    fn not(self) -> Self::Output {
        match self {
            Value::U64(v) => Value::U64(!v),
            Value::Bits(v) => Value::Bits(!v),
            Value::Bool(v) => Value::Bool(!v),
            _ => panic!("Cannot not non-integral value"),
        }
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Self::Output {
        match self {
            Value::U64(v) => Value::U64(v.wrapping_neg()),
            Value::Bits(v) => Value::Bits(v.wrapping_neg()),
            Value::Bool(v) => Value::Bool(!v),
            _ => panic!("Cannot negate non-integral value"),
        }
    }
}

impl From<u64> for Value {
    fn from(val: u64) -> Self {
        Value::U64(val)
    }
}
impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Value::Bool(val)
    }
}
impl From<Bits> for Value {
    fn from(val: Bits) -> Self {
        Value::Bits(val)
    }
}
impl From<String> for Value {
    fn from(val: String) -> Self {
        Value::String(val)
    }
}
impl From<EnumReference> for Value {
    fn from(val: EnumReference) -> Self {
        Value::EnumReference(val.0)
    }
}
impl From<&str> for Value {
    fn from(val: &str) -> Self {
        Value::String(val.into())
    }
}
impl From<Reference> for Value {
    fn from(val: Reference) -> Self {
        Value::Reference(val)
    }
}
impl From<PrecedenceType> for Value {
    fn from(val: PrecedenceType) -> Self {
        Value::PrecedenceType(val)
    }
}
impl From<AccessType> for Value {
    fn from(val: AccessType) -> Self {
        Value::AccessType(val)
    }
}
impl From<OnReadType> for Value {
    fn from(val: OnReadType) -> Self {
        Value::OnReadType(val)
    }
}
impl From<OnWriteType> for Value {
    fn from(val: OnWriteType) -> Self {
        Value::OnWriteType(val)
    }
}
impl From<AddressingType> for Value {
    fn from(val: AddressingType) -> Self {
        Value::AddressingType(val)
    }
}
impl From<InterruptType> for Value {
    fn from(val: InterruptType) -> Self {
        Value::InterruptType(val)
    }
}
impl TryFrom<Value> for u64 {
    type Error = RdlError<'static>;
    fn try_from(value: Value) -> Result<'static, Self> {
        match value {
            Value::U64(value) => Ok(value),
            _ => Err(RdlError::UnexpectedPropertyType {
                expected_type: PropertyType::U64,
                value,
            }),
        }
    }
}
impl TryFrom<Value> for bool {
    type Error = RdlError<'static>;
    fn try_from(value: Value) -> Result<'static, Self> {
        match value {
            Value::Bool(value) => Ok(value),
            _ => Err(RdlError::UnexpectedPropertyType {
                expected_type: PropertyType::Boolean,
                value,
            }),
        }
    }
}
impl TryFrom<Value> for Bits {
    type Error = RdlError<'static>;
    fn try_from(value: Value) -> Result<'static, Self> {
        match value {
            Value::Bits(value) => Ok(value),
            _ => Err(RdlError::UnexpectedPropertyType {
                expected_type: PropertyType::Bits,
                value,
            }),
        }
    }
}
impl TryFrom<Value> for String {
    type Error = RdlError<'static>;
    fn try_from(value: Value) -> Result<'static, Self> {
        match value {
            Value::String(value) => Ok(value),
            _ => Err(RdlError::UnexpectedPropertyType {
                expected_type: PropertyType::String,
                value,
            }),
        }
    }
}
impl TryFrom<Value> for EnumReference {
    type Error = RdlError<'static>;
    fn try_from(value: Value) -> Result<'static, Self> {
        match value {
            Value::EnumReference(value) => Ok(EnumReference(value)),
            _ => Err(RdlError::UnexpectedPropertyType {
                expected_type: PropertyType::String,
                value,
            }),
        }
    }
}
impl TryFrom<Value> for AddressingType {
    type Error = RdlError<'static>;
    fn try_from(value: Value) -> Result<'static, Self> {
        match value {
            Value::AddressingType(value) => Ok(value),
            _ => Err(RdlError::UnexpectedPropertyType {
                expected_type: PropertyType::AddressingType,
                value,
            }),
        }
    }
}
impl TryFrom<Value> for AccessType {
    type Error = RdlError<'static>;
    fn try_from(value: Value) -> Result<'static, Self> {
        match value {
            Value::AccessType(value) => Ok(value),
            _ => Err(RdlError::UnexpectedPropertyType {
                expected_type: PropertyType::AccessType,
                value,
            }),
        }
    }
}

pub fn parse_str_literal(s: &str) -> Result<String> {
    if s.len() < 2 || !s.starts_with('"') || !s.ends_with('"') {
        return Err(RdlError::BadStringLiteral);
    }
    Ok(s[1..s.len() - 1]
        .replace("\\\"", "\"")
        .replace("\\\\", "\\"))
}

fn to_bool<'a>(v: Value, parameters: Option<&'_ ParameterScope<'_>>) -> Result<'a, bool> {
    match v {
        Value::Bool(b) => Ok(b),
        Value::Reference(r) => {
            let r = r.path[0].clone();
            match lookup_parameter_of_type(parameters, &r, PropertyType::Boolean) {
                Ok(Value::Bool(b)) => Ok(*b),
                _ => Err(RdlError::UnexpectedPropertyType {
                    expected_type: PropertyType::Boolean,
                    value: Value::Bool(false),
                }),
            }
        }
        _ => Err(RdlError::UnexpectedPropertyType {
            expected_type: PropertyType::Boolean,
            value: v,
        }),
    }
}

fn to_bit<'a>(v: Value, parameters: Option<&'_ ParameterScope<'_>>) -> Result<'a, Bits> {
    match v {
        Value::Bits(b) => Ok(b),
        Value::Reference(r) => {
            let r = r.path[0].clone();
            match lookup_parameter_of_type(parameters, &r, PropertyType::Bits) {
                Ok(Value::Bits(b)) => Ok(*b),
                _ => Err(RdlError::UnexpectedPropertyType {
                    expected_type: PropertyType::Bits,
                    value: Value::Bool(false),
                }),
            }
        }
        _ => Err(RdlError::UnexpectedPropertyType {
            expected_type: PropertyType::Bits,
            value: v,
        }),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyType {
    U64,
    Bits,
    Boolean,
    BooleanOrReference,
    BitOrReference,
    EnumReference,
    // has posedge | negedge | bothedge | level | nonsticky modifiers
    FieldInterrupt,
    PrecedenceType,
    String,
    AccessType,
    InstanceOrPropRef,
    OnReadType,
    OnWriteType,
    AddressingType,
}
