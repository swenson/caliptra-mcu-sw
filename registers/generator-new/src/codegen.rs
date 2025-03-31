// Licensed under the Apache-2.0 license

use anyhow::bail;
use mcu_registers_systemrdl_new::ast::{
    ArrayOrRange, BinaryOp, ComponentBody, ComponentBodyElem, ComponentDef, ComponentInsts,
    ComponentType, ConstantExpr, ConstantExprContinue, ConstantPrimary, ConstantPrimaryBase,
    Description, EnumDef, ExplicitOrDefaultPropAssignment, ExplicitPropertyAssignment,
    IdentityOrPropKeyword, PrimaryLiteral, PropAssignmentRhs, PropertyAssignment, Root, UnaryOp,
};
use mcu_registers_systemrdl_new::FsFileSource;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

#[allow(unused)]
pub fn generate_tock_registers(input: &str, _addrmaps: &[&str]) -> anyhow::Result<String> {
    let root = mcu_registers_systemrdl_new::parse(input)?;
    Ok(format!("{:?}", root))
}

fn enumerate_instances(root: &Root, body: &ComponentBody) {
    println!("Enumerating instances for root");
}

struct RootRoot {
    children: Vec<Rc<dyn Component>>,
    enums: Vec<Enum>,
}

#[derive(Clone)]
struct Field {
    parent: Option<Rc<dyn Component>>,
    name: Option<String>,
    properties: HashMap<String, StringOrInt>,
    _fields: HashMap<String, Rc<Field>>, // just a placeholder
}

impl Component for Field {
    fn as_field(&self) -> Option<&Field> {
        Some(self)
    }
    fn component_type(&self) -> ComponentType {
        ComponentType::Field
    }
    fn parent(&self) -> Option<Rc<dyn Component>> {
        self.parent.clone()
    }
    fn fields(&self) -> &HashMap<String, Rc<Field>> {
        &self._fields
    }
    fn width(&self) -> usize {
        0
    }

    fn offset(&self) -> usize {
        0
    }

    fn instances(&self) -> &[RegisterInstance] {
        &[]
    }

    fn children(&self) -> &[Rc<dyn Component>] {
        &[]
    }
    fn enums(&self) -> &[Enum] {
        &[]
    }
    fn properties(&self) -> &HashMap<String, StringOrInt> {
        &self.properties
    }
}

#[derive(Clone, Debug)]
struct FieldInstance {
    id: String,
    offset: usize,
    width: usize,
}

fn convert_field(
    parent: Option<Rc<dyn Component>>,
    name: Option<&str>,
    body: &ComponentBody,
) -> Result<(Field, Vec<FieldInstance>), anyhow::Error> {
    let mut instances = vec![];
    println!("Field {:?}", name);
    for elem in body.elements.iter() {
        match elem {
            ComponentBodyElem::PropertyAssignment(pa) => {
                //println!("Property assignment: {:?}", pa);
                if let Some((key, value)) = evaluate_property(pa) {
                    println!("Property {}: {}", key, value);
                }
            }
            _ => todo!(),
        }
    }
    Ok((
        Field {
            parent,
            name: name.map(|s| s.to_string()),
            properties: HashMap::new(),
            _fields: HashMap::new(),
        },
        instances,
    ))
}

#[derive(Clone)]
struct Register {
    parent: Option<Rc<dyn Component>>,
    name: String,
    fields: HashMap<String, Rc<Field>>,
    field_instances: Vec<FieldInstance>,
    enums: Vec<Enum>,
    properties: HashMap<String, StringOrInt>,
}

impl std::fmt::Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Register {{ field_instances: {:?}, properties: {:?} }}",
            self.field_instances, self.properties
        )
    }
}

impl Component for Register {
    fn component_type(&self) -> ComponentType {
        ComponentType::Reg
    }
    fn parent(&self) -> Option<Rc<dyn Component>> {
        self.parent.clone()
    }
    fn fields(&self) -> &HashMap<String, Rc<Field>> {
        &self.fields
    }
    fn width(&self) -> usize {
        0
    }

    fn offset(&self) -> usize {
        0
    }

    fn instances(&self) -> &[RegisterInstance] {
        &[]
    }

    fn children(&self) -> &[Rc<dyn Component>] {
        &[]
    }
    fn enums(&self) -> &[Enum] {
        &[]
    }
    fn properties(&self) -> &HashMap<String, StringOrInt> {
        &self.properties
    }
}

#[derive(Clone, Debug)]
enum StringOrInt {
    String(String),
    Int(Integer),
}

impl std::fmt::Display for StringOrInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringOrInt::String(s) => write!(f, "{}", s),
            StringOrInt::Int(i) => write!(f, "{}", i.value),
        }
    }
}

fn convert_reg(
    parent: Option<Rc<dyn Component>>,
    name: &str,
    body: &ComponentBody,
) -> Result<Register, anyhow::Error> {
    //println!("Reg {} body: {:?}", name, body);

    let mut reg = Register {
        parent: parent.clone(),
        name: name.to_string(),
        fields: HashMap::new(),
        field_instances: vec![],
        enums: vec![],
        properties: HashMap::new(),
    };
    for elem in body.elements.iter() {
        match elem {
            ComponentBodyElem::PropertyAssignment(pa) => {
                if let Some((key, value)) = evaluate_property(pa) {
                    reg.properties.insert(key, value);
                }
            }
            ComponentBodyElem::ComponentDef(component) => {
                let comp = convert_component_field(parent.clone(), component)?;
                if let Some(comp) = comp {
                    if let Some(name) = comp.name.as_ref() {
                        println!("\nInserting field {} into reg", name);
                        reg.fields.insert(name.clone(), comp.clone());
                    }
                    if component.insts.is_some() {
                        let new_insts = convert_field_instances(
                            comp.clone(),
                            component.insts.as_ref().unwrap(),
                        )?;
                        reg.field_instances.extend(new_insts);
                    }
                }
            }
            ComponentBodyElem::EnumDef(enum_def) => {
                let enum_ = parse_enum(enum_def)?;
                reg.enums.push(enum_);
            }
            ComponentBodyElem::ExplicitComponentInst(inst) => {
                if let Some(field) = find_field(&reg, &inst.id) {
                    let new_insts = convert_field_instances(field, &inst.component_insts)?;
                    reg.field_instances.extend(new_insts);
                } else {
                    bail!("Field {} not found in scope", inst.id);
                }
            }
            _ => {
                println!("Unsupported element in register body: {:?}", elem);
                todo!()
            }
        }
    }
    Ok(reg)
}

fn find_field(scope: &dyn Component, name: &str) -> Option<Rc<Field>> {
    if let Some(f) = scope.fields().get(name) {
        Some(f.clone())
    } else if let Some(parent) = scope.parent() {
        find_field(parent.as_ref(), name)
    } else {
        None
    }
}

fn convert_field_instances(
    field: Rc<Field>,
    insts: &ComponentInsts,
) -> Result<Vec<FieldInstance>, anyhow::Error> {
    let mut instances = vec![];
    let mut last_msb: Option<u64> = None;
    for inst in insts.component_insts.iter() {
        let fieldwidth = if let Some(ArrayOrRange::Array(expr)) = inst.array_or_range.as_ref() {
            if expr.len() != 1 {
                bail!("Only single dimension arrays supported for field instances");
            }
            let x = &expr[0];
            evaluate_constant_expr_int(x)?.value
        } else {
            1 // TODO: support fieldwidth default property
        };
        let lsb = if let Some(eq) = inst.equals.as_ref() {
            evaluate_constant_expr_int(eq)?.value
        } else {
            last_msb.map(|b| b + 1).unwrap_or(0)
        };
        let msb = lsb + fieldwidth - 1;
        last_msb = Some(msb);
        let instance = FieldInstance {
            id: inst.id.clone(),
            width: fieldwidth as usize,
            offset: lsb as usize,
        };
        instances.push(instance);
    }
    Ok(instances)
}

fn convert_component(
    parent: Option<Rc<dyn Component>>,
    component: &mcu_registers_systemrdl_new::ast::Component,
) -> Result<Option<Rc<dyn Component>>, anyhow::Error> {
    match &component.def {
        ComponentDef::Named(t, name, _, body) => match *t {
            ComponentType::AddrMap => {
                let addrmap = convert_addrmap(parent, name, body)?;
                Ok(Some(Rc::new(addrmap)))
            }
            ComponentType::Signal => Ok(None),
            ComponentType::Field => {
                let (field, _insts) = convert_field(parent, Some(name), body)?;
                Ok(Some(Rc::new(field)))
            }
            _ => bail!("Unsupported named component type: {:?}", t),
        },
        ComponentDef::Anon(t, body) => match *t {
            ComponentType::AddrMap => {
                let addrmap = convert_addrmap(parent, "anon", body)?;
                Ok(Some(Rc::new(addrmap)))
            }
            ComponentType::Signal => Ok(None),
            ComponentType::Reg => {
                let reg = convert_reg(parent, "anon", body)?;
                //println!("Reg: {:?}", reg);
                Ok(Some(Rc::new(reg)))
            }
            _ => bail!("Unsupported component type: {:?}", t),
        },
    }
}

fn convert_component_field(
    parent: Option<Rc<dyn Component>>,
    component: &mcu_registers_systemrdl_new::ast::Component,
) -> Result<Option<Rc<Field>>, anyhow::Error> {
    match &component.def {
        ComponentDef::Named(t, name, _, body) => match *t {
            ComponentType::Field => {
                let (field, _insts) = convert_field(parent, Some(name), body)?;
                Ok(Some(Rc::new(field)))
            }
            _ => bail!("Unsupported named component type: {:?}", t),
        },
        ComponentDef::Anon(t, body) => match *t {
            ComponentType::Field => {
                let (field, _insts) = convert_field(parent, None, body)?;
                Ok(Some(Rc::new(field)))
            }
            _ => bail!("Unsupported component type: {:?}", t),
        },
    }
}

fn convert_addrmap(
    parent: Option<Rc<dyn Component>>,
    name: &str,
    body: &ComponentBody,
) -> Result<AddrMap, anyhow::Error> {
    let mut enums = vec![];
    let mut children = vec![];
    let mut properties = HashMap::new();
    let mut instances = vec![];
    let mut fields = HashMap::new();
    for elem in body.elements.iter() {
        match elem {
            ComponentBodyElem::ComponentDef(component) => {
                let comp = convert_component(parent.clone(), component)?;
                if let Some(comp) = comp {
                    if component.insts.is_some() && comp.component_type() == ComponentType::Reg {
                        let new_insts =
                            convert_instances(comp.clone(), component.insts.as_ref().unwrap())?;
                        instances.extend(new_insts);
                    }
                    children.push(comp.clone());
                    comp.clone().as_field().map(|f| {
                        if let Some(name) = &f.name {
                            println!("\nInserting field {} into map", name);
                            fields.insert(name.clone(), Rc::new(f.clone()));
                        }
                    });
                }
            }
            ComponentBodyElem::EnumDef(enum_def) => {
                enums.push(parse_enum(enum_def)?);
            }
            ComponentBodyElem::StructDef(struct_def) => todo!(),
            ComponentBodyElem::ConstraintDef(constraint_def) => todo!(),
            ComponentBodyElem::ExplicitComponentInst(explicit_component_inst) => todo!(),
            ComponentBodyElem::PropertyAssignment(property_assignment) => {
                //println!("Property assignment: {:?}", property_assignment);
                if let Some((key, value)) = evaluate_property(property_assignment) {
                    properties.insert(key, value);
                }
            }
        }
    }
    println!("Properties {}: {:?}", name, properties);
    Ok(AddrMap {
        name: name.to_string(),
        offset: 0,
        width: 0,
        parent,
        children,
        instances: vec![],
        enums,
        properties,
        fields,
    })
}

fn convert_instances(
    reg: Rc<dyn Component>,
    insts: &ComponentInsts,
) -> Result<Vec<RegisterInstance>, anyhow::Error> {
    // println!(
    //     "Converting instances for reg {:?}: {:?}",
    //     reg.component_type(),
    //     insts
    // );
    let mut instances = vec![];
    for inst in insts.component_insts.iter() {
        let offset = if let Some(eq) = &inst.equals {
            Some(evaluate_constant_expr_int(&eq)?.value as usize)
        } else {
            None
        };
        // TODO: support regwidth
        let inst = RegisterInstance {
            name: inst.id.clone(),
            offset,
            width: 32,
            type_: reg.clone(),
        };
        instances.push(inst);
    }
    Ok(instances)
}

fn evaluate_property(property_assignment: &PropertyAssignment) -> Option<(String, StringOrInt)> {
    match property_assignment {
        PropertyAssignment::ExplicitOrDefaultPropAssignment(pa) => match pa {
            ExplicitOrDefaultPropAssignment::ExplicitPropModifier(
                _default_keyword,
                _explicit_prop_modifier,
            ) => todo!(),
            ExplicitOrDefaultPropAssignment::ExplicitPropAssignment(_default, epa) => match epa {
                ExplicitPropertyAssignment::Assignment(
                    identity_or_prop_keyword,
                    prop_assignment_rhs,
                ) => {
                    let id = match identity_or_prop_keyword {
                        IdentityOrPropKeyword::Id(id) => id.clone(),
                        IdentityOrPropKeyword::PropKeyword(prop_keyword) => {
                            prop_keyword.to_string()
                        }
                    };
                    let rhs = match prop_assignment_rhs {
                        Some(rhs) => match rhs {
                            PropAssignmentRhs::ConstantExpr(constant_expr) => {
                                evaluate_constant_expr_str(constant_expr)
                                    .ok()
                                    .map(StringOrInt::String)
                                    .or(evaluate_constant_expr_int(constant_expr)
                                        .map(StringOrInt::Int)
                                        .ok())
                            }
                            PropAssignmentRhs::PrecedenceType(_precedence_type) => todo!(),
                        },
                        None => todo!(),
                    };
                    rhs.map(|rhs| (id.clone(), rhs))
                }
                ExplicitPropertyAssignment::EncodeAssignment(e) => {
                    Some(("encode".to_string(), StringOrInt::String(e.clone())))
                }
            },
        },
        PropertyAssignment::PostPropAssignment(_post_prop_assignment) => todo!(),
    }
}

const TRUE: Integer = Integer { width: 1, value: 1 };

const FALSE: Integer = Integer { width: 1, value: 0 };

fn evaluate_constant_expr_str(expr: &ConstantExpr) -> Result<String, anyhow::Error> {
    match expr {
        ConstantExpr::ConstantPrimary(prim, cont) => {
            if cont.is_some() {
                bail!("Unsupported complex expression for string");
            }
            match prim {
                ConstantPrimary::Base(constant_primary_base) => match constant_primary_base {
                    ConstantPrimaryBase::PrimaryLiteral(primary_literal) => match primary_literal {
                        PrimaryLiteral::StringLiteral(s) => Ok(s.clone()),
                        _ => bail!(
                            "Unsupported literal in string evaluation context: {:?}",
                            primary_literal
                        ),
                    },
                    _ => {
                        bail!("Unsupported expression for string");
                    }
                },
                ConstantPrimary::Cast(constant_primary_base, constant_expr) => {
                    bail!("Casting string not supported")
                }
            }
        }
        ConstantExpr::UnaryOp(op, expr, cont) => {
            bail!("Unsupported unary operation on string: {:?}", op);
        }
    }
}

fn evaluate_constant_expr_cont_int(
    val: Integer,
    cont: &Option<Box<ConstantExprContinue>>,
) -> Result<Integer, anyhow::Error> {
    match cont {
        None => Ok(val),
        Some(cont) => {
            match cont.as_ref() {
                ConstantExprContinue::BinaryOp(op, expr, cont) => {
                    let rhs = evaluate_constant_expr_int(expr.as_ref())?;

                    let a = val.value;
                    let b = rhs.value;
                    let width = val.width;

                    // short circuit for shift since they may have different widths
                    let new_val = match op {
                        BinaryOp::RightShift => Some(a >> b),
                        BinaryOp::LeftShift => Some(a << b),
                        _ => None,
                    };
                    if let Some(value) = new_val {
                        return Ok(Integer { width, value });
                    }

                    if val.width != rhs.width {
                        bail!(
                            "Incompatible widths in expression: {} and {}",
                            val.width,
                            rhs.width
                        );
                    }

                    // Check booleans
                    let bool_value = match op {
                        BinaryOp::LessThan => Some(if a < b { TRUE } else { FALSE }),
                        BinaryOp::GreaterThan => Some(if a > b { TRUE } else { FALSE }),
                        BinaryOp::LessThanOrEqual => Some(if a <= b { TRUE } else { FALSE }),
                        BinaryOp::GreaterThanOrEqual => Some(if a >= b { TRUE } else { FALSE }),
                        BinaryOp::EqualsEquals => Some(if a == b { TRUE } else { FALSE }),
                        BinaryOp::NotEquals => Some(if a != b { TRUE } else { FALSE }),
                        _ => None,
                    };

                    if let Some(b) = bool_value {
                        return Ok(b);
                    }

                    let value: u64 = match op {
                        BinaryOp::AndAnd => a & b,
                        BinaryOp::OrOr => a | b,
                        BinaryOp::And => a & b,
                        BinaryOp::Or => a | b,
                        BinaryOp::Xor => a ^ b,
                        BinaryOp::Xnor => !(a ^ b),
                        BinaryOp::Times => a * b,
                        BinaryOp::Divide => a / b,
                        BinaryOp::Modulus => a % b,
                        BinaryOp::Plus => a + b,
                        BinaryOp::Minus => a - b,
                        BinaryOp::Power => a.pow(b as u32),
                        _ => unreachable!(),
                    };
                    Ok(Integer { width, value })
                }
                ConstantExprContinue::TernaryOp(b, c, cont) => {
                    let a = val;
                    if a.width != 1 {
                        bail!("Cannot use non-boolean value as ternary condition");
                    }
                    let b = evaluate_constant_expr_int(b.as_ref())?;
                    let c = evaluate_constant_expr_int(c.as_ref())?;
                    if a == TRUE {
                        evaluate_constant_expr_cont_int(b, cont)
                    } else {
                        evaluate_constant_expr_cont_int(c, cont)
                    }
                }
            }
        }
    }
}

fn evaluate_primary_literal_int(p: &PrimaryLiteral) -> Result<Integer, anyhow::Error> {
    let value = match p {
        PrimaryLiteral::Number(n) => Integer {
            width: 32,
            value: *n,
        },
        PrimaryLiteral::Bits(b) => Integer {
            width: b.w() as u8,
            value: b.val(),
        },
        _ => bail!("Unsupported literal in integer evaluation context: {:?}", p),
    };
    Ok(value)
}

fn evaluate_constant_primary_base_int(
    base: &ConstantPrimaryBase,
) -> Result<Integer, anyhow::Error> {
    match base {
        ConstantPrimaryBase::PrimaryLiteral(p) => evaluate_primary_literal_int(p),
        ConstantPrimaryBase::ConstantExpr(c) => evaluate_constant_expr_int(c),
        ConstantPrimaryBase::InstanceOrPropRef(_) => {
            bail!("References not supported in integer context")
        }
        ConstantPrimaryBase::StructLiteral(_, _) => {
            bail!("Struct literal not supported in integer context")
        }
        ConstantPrimaryBase::ArrayLiteral(_) => {
            bail!("Array literal not supported in integer context")
        }
        ConstantPrimaryBase::SimpleTypeCast(_, _) => {
            bail!("Simple type cast not supported in integer context")
        }
        ConstantPrimaryBase::BooleanCast(_) => {
            bail!("Boolean type cast not supported in integer context")
        }
        ConstantPrimaryBase::ConstantConcat(_) => bail!("Integer concatenation not supported"),
        ConstantPrimaryBase::ConstantMultipleConcat(_, _) => {
            bail!("Integer multiple concatenation not supported")
        }
    }
}

fn evaluate_cast(value: Integer, expr: &ConstantExpr) -> Result<Integer, anyhow::Error> {
    bail!("Casting not supported");
}

fn evaluate_constant_primary_int(prim: &ConstantPrimary) -> Result<Integer, anyhow::Error> {
    match prim {
        ConstantPrimary::Base(base) => evaluate_constant_primary_base_int(base),
        ConstantPrimary::Cast(base, cast) => {
            let base = evaluate_constant_primary_base_int(base)?;
            evaluate_cast(base, cast.as_ref())
        }
    }
}

fn evaluate_constant_expr_int(expr: &ConstantExpr) -> Result<Integer, anyhow::Error> {
    match expr {
        ConstantExpr::ConstantPrimary(prim, cont) => {
            let val = evaluate_constant_primary_int(prim)?;
            evaluate_constant_expr_cont_int(val, cont)
        }
        ConstantExpr::UnaryOp(op, expr, cont) => {
            let expr = evaluate_constant_expr_int(expr)?;
            let width = expr.width;
            let val = expr.value;
            let new_val = match op {
                UnaryOp::LogicalNot => !val,
                UnaryOp::Plus => val,
                UnaryOp::Minus => (!val) + 1,
                UnaryOp::Not => !val,
                UnaryOp::And => bail!("Unsupported unary operation on integer: &"),
                UnaryOp::Nand => bail!("Unsupported unary operation on integer: ~&"),
                UnaryOp::Or => bail!("Unsupported unary operation on integer: |"),
                UnaryOp::Nor => bail!("Unsupported unary operation on integer: ~&"),
                UnaryOp::Xor => bail!("Unsupported unary operation on integer: ^"),
                UnaryOp::Xnor => bail!("Unsupported unary operation on integer: &"),
            };
            let val = Integer {
                width,
                value: new_val,
            };
            evaluate_constant_expr_cont_int(val, cont)
        }
    }
}

fn parse_enum(e: &EnumDef) -> Result<Enum, anyhow::Error> {
    let mut values = vec![];
    let mut last_value: Option<Integer> = None;
    for entry in e.body.iter() {
        let val = match (&last_value, &entry.expr) {
            (None, None) => Integer {
                width: 32,
                value: 0,
            },
            (Some(last_val), None) => last_val.add(1),
            (_, Some(expr)) => evaluate_constant_expr_int(expr)?,
        };
        last_value = Some(val);
        let val = EnumValue {
            name: entry.id.clone(),
            value: val,
        };
        values.push(val);
    }
    Ok(Enum {
        name: e.id.clone(),
        values,
    })
}

#[allow(dead_code)]
fn parse_root(root: &Root) -> Result<RootRoot, anyhow::Error> {
    let mut root_root = RootRoot {
        children: Vec::new(),
        enums: Vec::new(),
    };
    for d in root.descriptions.iter() {
        match d {
            Description::EnumDef(e) => {
                root_root.enums.push(parse_enum(e)?);
            }
            Description::ComponentDef(c) => match &c.def {
                ComponentDef::Named(t, name, _, body) => {
                    match *t {
                        ComponentType::AddrMap => {
                            let addrmap = convert_addrmap(None, name, body)?;
                            root_root.children.push(Rc::new(addrmap));
                        }
                        // ComponentType::Reg => {
                        //     let reg = convert_reg(None, name, body);
                        //     root_root.children.push(Rc::new(reg));
                        // }
                        // ComponentType::RegFile => {
                        //     let regfile = convert_regfile(None, name, body);
                        //     root_root.children.push(Rc::new(regfile));
                        // }
                        _ => {
                            println!("Other component type: {:?}", t);
                        }
                    }
                    // if *t == ComponentType::AddrMap {
                    //     println!("Component {:?} {}", t, name);
                    // } else if *t == ComponentType::
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(root_root)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Integer {
    width: u8, // Verilog supports larger numbers, but we don't
    value: u64,
}

impl Integer {
    fn add(&self, val: u64) -> Integer {
        Integer {
            width: self.width,
            value: self.value + val,
        }
    }
}

#[derive(Clone, Debug)]
struct EnumValue {
    name: String,
    value: Integer,
}

#[derive(Clone, Debug)]
struct Enum {
    name: String,
    values: Vec<EnumValue>,
}

trait Component {
    fn as_field(&self) -> Option<&Field> {
        None
    }
    fn component_type(&self) -> ComponentType;
    fn parent(&self) -> Option<Rc<dyn Component>>;
    fn width(&self) -> usize;
    fn offset(&self) -> usize;
    fn fields(&self) -> &HashMap<String, Rc<Field>>;
    fn instances(&self) -> &[RegisterInstance];

    fn children(&self) -> &[Rc<dyn Component>];
    fn enums(&self) -> &[Enum];
    fn properties(&self) -> &HashMap<String, StringOrInt>;
}

struct RegisterInstance {
    name: String,
    offset: Option<usize>,
    width: usize,
    type_: Rc<dyn Component>,
}

struct AddrMap {
    name: String,
    offset: usize,
    width: usize,
    parent: Option<Rc<dyn Component>>,
    children: Vec<Rc<dyn Component>>,
    fields: HashMap<String, Rc<Field>>,
    instances: Vec<RegisterInstance>,
    enums: Vec<Enum>,
    properties: HashMap<String, StringOrInt>,
}

impl Component for AddrMap {
    fn component_type(&self) -> ComponentType {
        ComponentType::AddrMap
    }
    fn parent(&self) -> Option<Rc<dyn Component>> {
        self.parent.clone()
    }
    fn fields(&self) -> &HashMap<String, Rc<Field>> {
        &self.fields
    }
    fn width(&self) -> usize {
        self.width
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn instances(&self) -> &[RegisterInstance] {
        &self.instances
    }

    fn children(&self) -> &[Rc<dyn Component>] {
        &self.children
    }

    fn enums(&self) -> &[Enum] {
        &self.enums
    }

    fn properties(&self) -> &HashMap<String, StringOrInt> {
        &self.properties
    }
}

pub fn generate_tock_registers_from_file(file: &Path, addrmaps: &[&str]) -> anyhow::Result<String> {
    let src = FsFileSource::new();
    let root = Root::from_file(&src, file)?;
    println!("Found {} descriptions", root.descriptions.len());

    let root_root = parse_root(&root)?;
    for d in root.descriptions.iter() {
        match d {
            Description::ComponentDef(c) => match &c.def {
                ComponentDef::Named(t, name, _, body) => {
                    if *t == ComponentType::AddrMap && addrmaps.contains(&&**name) {
                        println!("Component {:?} {}", t, name);

                        enumerate_instances(&root, body);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok("".to_string())
}

#[cfg(test)]
mod test {
    use super::{generate_tock_registers, generate_tock_registers_from_file};
    use std::path::Path;

    #[test]
    fn test_mcu() {
        let result = generate_tock_registers_from_file(
            Path::new("/home/chswenson/work/mcu-sw/hw/mcu.rdl"),
            &["mcu"],
        )
        .unwrap();
        println!("{}", result);
    }

    #[test]
    fn test() {
        let result = generate_tock_registers(
            r#"
addrmap mcu {
    I3CCSR I3CCSR @ 0x2000_4000;
    mci_top mci_top @ 0x2100_0000;
};
"#,
            &[],
        )
        .unwrap();
        println!("{}", result);
    }
}
