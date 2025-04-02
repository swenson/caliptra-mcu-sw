// Licensed under the Apache-2.0 license

use anyhow::bail;
use mcu_registers_systemrdl_new::ast::{
    ArrayOrRange, BinaryOp, ComponentBody, ComponentBodyElem, ComponentInsts, ComponentType,
    ConstantExpr, ConstantExprContinue, ConstantPrimary, ConstantPrimaryBase, Description, EnumDef,
    ExplicitOrDefaultPropAssignment, ExplicitPropertyAssignment, IdentityOrPropKeyword,
    PrimaryLiteral, PropAssignmentRhs, PropertyAssignment, Root, UnaryOp,
};
use mcu_registers_systemrdl_new::FsFileSource;
use std::collections::HashMap;
use std::path::Path;

#[allow(unused)]
pub fn generate_tock_registers(input: &str, _addrmaps: &[&str]) -> anyhow::Result<String> {
    let root = mcu_registers_systemrdl_new::parse(input)?;
    Ok(format!("{:?}", root))
}

fn enumerate_instances(_root: &Root, _body: &ComponentBody) {
    println!("Enumerating instances for root");
}

const TRUE: Integer = Integer { width: 1, value: 1 };
const FALSE: Integer = Integer { width: 1, value: 0 };

#[derive(Clone, Default)]
struct World {
    /// List of component children.
    child_components: Vec<ComponentIdx>,
    /// List of instance children.
    child_instances: Vec<InstanceIdx>,
    enums: Vec<Enum>,
    /// Holds all of the components so that they can be referenced by index.
    /// They can be added but never deleted.
    component_arena: Vec<AllComponent>,
    /// Holds all of the instances so that they can be referenced by index.
    /// They can be added but never deleted.
    instance_arena: Vec<Instance>,
}

type ComponentIdx = usize;
type InstanceIdx = usize;

#[derive(Clone)]
enum AllComponent {
    AddrMap(AddrMapType),
    Reg(RegisterType),
    RegFile(RegisterFileType),
    Field(FieldType),
}

impl AllComponent {
    fn as_addrmap(&mut self) -> Option<&mut AddrMapType> {
        if let AllComponent::AddrMap(addrmap) = self {
            Some(addrmap)
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct Instance {
    name: String,
    offset: usize,
    width: usize,
    desc: Option<String>,
    array_size: Option<Vec<usize>>,
    type_idx: ComponentIdx,
    parent: Option<ComponentIdx>,
    children: Vec<InstanceIdx>,
}

impl Component for AllComponent {
    fn name(&self) -> Option<&str> {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.name(),
            AllComponent::Reg(reg) => reg.name(),
            AllComponent::RegFile(regfile) => regfile.name(),
            AllComponent::Field(field) => field.name(),
        }
    }

    fn component_type(&self) -> ComponentType {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.component_type(),
            AllComponent::Reg(reg) => reg.component_type(),
            AllComponent::RegFile(regfile) => regfile.component_type(),
            AllComponent::Field(field) => field.component_type(),
        }
    }

    fn parent(&self) -> Option<ComponentIdx> {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.parent(),
            AllComponent::Reg(reg) => reg.parent(),
            AllComponent::RegFile(regfile) => regfile.parent(),
            AllComponent::Field(field) => field.parent(),
        }
    }

    fn width(&self) -> usize {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.width(),
            AllComponent::Reg(reg) => reg.width(),
            AllComponent::RegFile(regfile) => regfile.width(),
            AllComponent::Field(field) => field.width(),
        }
    }

    fn offset(&self) -> usize {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.offset(),
            AllComponent::Reg(reg) => reg.offset(),
            AllComponent::RegFile(regfile) => regfile.offset(),
            AllComponent::Field(field) => field.offset(),
        }
    }

    fn fields(&self) -> &HashMap<String, ComponentIdx> {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.fields(),
            AllComponent::Reg(reg) => reg.fields(),
            AllComponent::RegFile(regfile) => regfile.fields(),
            AllComponent::Field(field) => field.fields(),
        }
    }

    fn children(&self) -> &[ComponentIdx] {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.children(),
            AllComponent::Reg(reg) => reg.children(),
            AllComponent::RegFile(regfile) => regfile.children(),
            AllComponent::Field(field) => field.children(),
        }
    }

    fn enums(&self) -> &[Enum] {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.enums(),
            AllComponent::Reg(reg) => reg.enums(),
            AllComponent::RegFile(regfile) => regfile.enums(),
            AllComponent::Field(field) => field.enums(),
        }
    }

    fn properties(&self) -> &HashMap<String, StringOrInt> {
        match self {
            AllComponent::AddrMap(addrmap) => addrmap.properties(),
            AllComponent::Reg(reg) => reg.properties(),
            AllComponent::RegFile(regfile) => regfile.properties(),
            AllComponent::Field(field) => field.properties(),
        }
    }
}

#[derive(Clone)]
struct FieldType {
    parent: Option<ComponentIdx>,
    name: Option<String>,
    properties: HashMap<String, StringOrInt>,
    _fields: HashMap<String, ComponentIdx>, // just a placeholder
}

impl Component for FieldType {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    fn as_field(&self) -> Option<&FieldType> {
        Some(self)
    }
    fn component_type(&self) -> ComponentType {
        ComponentType::Field
    }
    fn parent(&self) -> Option<ComponentIdx> {
        self.parent
    }
    fn fields(&self) -> &HashMap<String, ComponentIdx> {
        &self._fields
    }
    fn width(&self) -> usize {
        0
    }

    fn offset(&self) -> usize {
        0
    }

    fn children(&self) -> &[ComponentIdx] {
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

#[derive(Clone)]
struct RegisterType {
    parent: Option<ComponentIdx>,
    name: Option<String>,
    fields: HashMap<String, ComponentIdx>,
    field_instances: Vec<FieldInstance>,
    enums: Vec<Enum>,
    properties: HashMap<String, StringOrInt>,
}

impl std::fmt::Debug for RegisterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Register {{ field_instances: {:?}, properties: {:?} }}",
            self.field_instances, self.properties
        )
    }
}

impl Component for RegisterType {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    fn component_type(&self) -> ComponentType {
        ComponentType::Reg
    }
    fn parent(&self) -> Option<ComponentIdx> {
        self.parent
    }
    fn fields(&self) -> &HashMap<String, ComponentIdx> {
        &self.fields
    }
    fn width(&self) -> usize {
        0
    }

    fn offset(&self) -> usize {
        0
    }

    fn children(&self) -> &[ComponentIdx] {
        &[]
    }
    fn enums(&self) -> &[Enum] {
        &[]
    }
    fn properties(&self) -> &HashMap<String, StringOrInt> {
        &self.properties
    }
}

#[derive(Clone)]
struct RegisterFileType {
    parent: Option<ComponentIdx>,
    name: String,
    fields: HashMap<String, ComponentIdx>,
    field_instances: Vec<FieldInstance>,
    enums: Vec<Enum>,
    properties: HashMap<String, StringOrInt>,
}

impl std::fmt::Debug for RegisterFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Register {{ field_instances: {:?}, properties: {:?} }}",
            self.field_instances, self.properties
        )
    }
}

impl Component for RegisterFileType {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
    fn component_type(&self) -> ComponentType {
        ComponentType::RegFile
    }
    fn parent(&self) -> Option<ComponentIdx> {
        self.parent
    }
    fn fields(&self) -> &HashMap<String, ComponentIdx> {
        &self.fields
    }
    fn width(&self) -> usize {
        0
    }

    fn offset(&self) -> usize {
        0
    }

    fn children(&self) -> &[ComponentIdx] {
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

impl World {
    pub fn parse(root: &Root) -> Result<Self, anyhow::Error> {
        let mut world = World::default();
        world.parse_root(root)?;
        Ok(world)
    }

    pub fn parse_root(&mut self, root: &Root) -> Result<(), anyhow::Error> {
        for d in root.descriptions.iter() {
            match d {
                Description::EnumDef(e) => {
                    let e = self.parse_enum(e)?;
                    self.enums.push(e);
                }
                Description::ComponentDef(c) => match c.def.type_ {
                    ComponentType::AddrMap => {
                        let name = c.def.name.as_deref().unwrap_or("anon");
                        self.add_addrmap(None, name, &c.def.body)?;
                    }
                    // ComponentType::Reg => {
                    //     let reg = convert_reg(None, name, body);
                    //     root_root.children.push(Rc::new(reg));
                    // }
                    // ComponentType::RegFile => {
                    //     let regfile = convert_regfile(None, name, body);
                    //     root_root.children.push(Rc::new(regfile));
                    // }
                    t => {
                        println!("Other component type: {:?}", t);
                    } // if *t == ComponentType::AddrMap {
                      //     println!("Component {:?} {}", t, name);
                      // } else if *t == ComponentType::
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn convert_field(
        &self,
        parent: Option<ComponentIdx>,
        name: Option<&str>,
        body: &ComponentBody,
    ) -> Result<(FieldType, Vec<FieldInstance>), anyhow::Error> {
        let instances = vec![];
        //println!("Field {:?}", name);
        for elem in body.elements.iter() {
            match elem {
                ComponentBodyElem::PropertyAssignment(pa) => {
                    //println!("Property assignment: {:?}", pa);
                    if let Some((key, value)) = self.evaluate_property(pa) {
                        //println!("Property {}: {}", key, value);
                    }
                }
                _ => todo!(),
            }
        }
        Ok((
            FieldType {
                parent,
                name: name.map(|s| s.to_string()),
                properties: HashMap::new(),
                _fields: HashMap::new(),
            },
            instances,
        ))
    }

    fn convert_reg(
        &mut self,
        parent: Option<ComponentIdx>,
        name: Option<&str>,
        body: &ComponentBody,
    ) -> Result<RegisterType, anyhow::Error> {
        //println!("Reg {} body: {:?}", name, body);

        let mut reg = RegisterType {
            parent,
            name: name.map(|name| name.to_string()),
            fields: HashMap::new(),
            field_instances: vec![],
            enums: vec![],
            properties: HashMap::new(),
        };
        for elem in body.elements.iter() {
            match elem {
                ComponentBodyElem::PropertyAssignment(pa) => {
                    if let Some((key, value)) = self.evaluate_property(pa) {
                        reg.properties.insert(key, value);
                    }
                }
                ComponentBodyElem::ComponentDef(component) => {
                    let comp = self.convert_component_field(parent, component)?;
                    if let Some(comp_idx) = comp {
                        let comp = &self.component_arena[comp_idx];
                        if let Some(name) = comp.name() {
                            println!("\nInserting field {} into reg", name);
                            reg.fields.insert(name.to_string(), comp_idx);
                        }
                        if component.insts.is_some() {
                            let new_insts = self.convert_field_instances(
                                comp_idx,
                                component.insts.as_ref().unwrap(),
                            )?;
                            reg.field_instances.extend(new_insts);
                        }
                    }
                    // TODO: check other kinds
                }
                ComponentBodyElem::EnumDef(enum_def) => {
                    let enum_ = self.parse_enum(enum_def)?;
                    reg.enums.push(enum_);
                }
                ComponentBodyElem::ExplicitComponentInst(inst) => {
                    // look in the register components first
                    if reg.fields.contains_key(&inst.id) {
                        let field_idx = reg.fields[&inst.id];
                        let new_insts =
                            self.convert_field_instances(field_idx, &inst.component_insts)?;
                        reg.field_instances.extend(new_insts);
                    } else if reg.enums.iter().find(|e| e.name == inst.id).is_some() {
                        // found in enums
                    } else if let Some(parent) = parent {
                        // find in parent
                        if let Some(component_idx) =
                            self.find_component(&self.component_arena[parent], &inst.id)
                        {
                            todo!()
                        } else {
                            todo!()
                        }
                    } else {
                        bail!("Component {} not found in scope", inst.id);
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

    fn convert_regfile(
        &mut self,
        parent: Option<ComponentIdx>,
        name: &str,
        body: &ComponentBody,
    ) -> Result<RegisterFileType, anyhow::Error> {
        //panic!("Regfile {} body: {:?}", name, body);
        let mut regfile = RegisterFileType {
            parent,
            name: name.to_string(),
            fields: HashMap::new(),
            field_instances: vec![],
            enums: vec![],
            properties: HashMap::new(),
        };
        for elem in body.elements.iter() {
            match elem {
                ComponentBodyElem::PropertyAssignment(pa) => {
                    if let Some((key, value)) = self.evaluate_property(pa) {
                        regfile.properties.insert(key, value);
                    }
                }
                ComponentBodyElem::ComponentDef(component) => {
                    println!("Component {:?} for regfile", component.def.name.as_deref());
                    let comp = self.convert_component_field(parent, component)?;
                    println!("Comp {:?}", comp);
                    if let Some(comp_idx) = comp {
                        let comp = &self.component_arena[comp_idx];
                        if let Some(name) = comp.name() {
                            println!("\nInserting field {} into regfile", name);
                            regfile.fields.insert(name.to_string(), comp_idx);
                        }
                        if component.insts.is_some() {
                            let new_insts = self.convert_field_instances(
                                comp_idx,
                                component.insts.as_ref().unwrap(),
                            )?;
                            regfile.field_instances.extend(new_insts);
                        }
                    }
                    let comp = self.convert_component_reg(parent, &component)?;
                    if let Some(comp_idx) = comp {
                        let comp = &self.component_arena[comp_idx];
                        if let Some(name) = comp.name() {
                            println!("\nInserting field {} into regfile", name);
                            regfile.fields.insert(name.to_string(), comp_idx);
                        }
                        if component.insts.is_some() {
                            let new_insts = self.convert_field_instances(
                                comp_idx,
                                component.insts.as_ref().unwrap(),
                            )?;
                            regfile.field_instances.extend(new_insts);
                        }
                    }
                }
                ComponentBodyElem::EnumDef(enum_def) => {
                    let enum_ = self.parse_enum(enum_def)?;
                    regfile.enums.push(enum_);
                }
                ComponentBodyElem::ExplicitComponentInst(inst) => {
                    // look in the register components first
                    if regfile.fields.contains_key(&inst.id) {
                        let field_idx = regfile.fields[&inst.id];
                        let new_insts =
                            self.convert_field_instances(field_idx, &inst.component_insts)?;
                        regfile.field_instances.extend(new_insts);
                    } else if regfile.enums.iter().find(|e| e.name == inst.id).is_some() {
                        // found in enums
                    } else if let Some(parent) = parent {
                        // find in parent
                        if let Some(component_idx) =
                            self.find_component(&self.component_arena[parent], &inst.id)
                        {
                            todo!()
                        } else {
                            todo!()
                        }
                    } else {
                        bail!("Component {} not found in regfile scope {}", inst.id, name);
                    }
                }
                _ => {
                    println!("Unsupported element in regfile body: {:?}", elem);
                    todo!()
                }
            }
        }
        Ok(regfile)
    }

    fn find_field(&self, src: &AllComponent, name: &str) -> Option<ComponentIdx> {
        if let Some(f) = src.fields().get(name) {
            Some(f.clone())
        } else if let Some(parent) = src.parent() {
            self.find_field(&self.component_arena[parent], name)
        } else {
            None
        }
    }

    fn convert_field_instances(
        &self,
        _field_idx: ComponentIdx,
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
                self.evaluate_constant_expr_int(x)?.value
            } else {
                1 // TODO: support fieldwidth default property
            };
            let lsb = if let Some(eq) = inst.equals.as_ref() {
                self.evaluate_constant_expr_int(eq)?.value
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
        &mut self,
        parent: Option<ComponentIdx>,
        component: &mcu_registers_systemrdl_new::ast::Component,
    ) -> Result<Option<ComponentIdx>, anyhow::Error> {
        let t = component.def.type_;
        let name = component.def.name.clone().unwrap_or("anon".to_string());
        let body = &component.def.body;
        match t {
            ComponentType::AddrMap => {
                let idx = self.add_addrmap(parent, &name, body)?;
                Ok(Some(idx))
            }
            ComponentType::Signal => Ok(None),
            ComponentType::Field => {
                let (field, _insts) =
                    self.convert_field(parent, component.def.name.as_deref(), body)?;
                self.component_arena.push(AllComponent::Field(field));
                Ok(Some(self.component_arena.len() - 1))
            }
            ComponentType::Reg => {
                let reg = self.convert_reg(None, Some(&name), body)?;
                self.component_arena.push(AllComponent::Reg(reg));
                Ok(Some(self.component_arena.len() - 1))
            }
            ComponentType::RegFile => {
                let regfile = self.convert_regfile(None, &name, body)?;
                self.component_arena.push(AllComponent::RegFile(regfile));
                Ok(Some(self.component_arena.len() - 1))
            }
            _ => bail!("Unsupported component type: {:?}", t),
        }
    }

    fn convert_component_field(
        &mut self,
        parent: Option<ComponentIdx>,
        component: &mcu_registers_systemrdl_new::ast::Component,
    ) -> Result<Option<ComponentIdx>, anyhow::Error> {
        let t = component.def.type_;
        let name = component.def.name.clone();
        let body = &component.def.body;
        match t {
            ComponentType::Field => {
                let (field, _insts) = self.convert_field(parent, name.as_deref(), body)?;
                // TODO: add field instances
                self.component_arena.push(AllComponent::Field(field));
                Ok(Some(self.component_arena.len() - 1))
            }
            _ => Ok(None),
        }
    }

    fn convert_component_reg(
        &mut self,
        parent: Option<ComponentIdx>,
        component: &mcu_registers_systemrdl_new::ast::Component,
    ) -> Result<Option<ComponentIdx>, anyhow::Error> {
        let t = component.def.type_;
        let name = component.def.name.clone();
        let body = &component.def.body;
        match t {
            ComponentType::Reg => {
                let reg = self.convert_reg(parent, name.as_deref(), body)?;
                // TODO: add reg instances
                self.component_arena.push(AllComponent::Reg(reg));
                Ok(Some(self.component_arena.len() - 1))
            }
            _ => Ok(None),
        }
    }

    fn find_component<'a>(&self, src: &AllComponent, name: &str) -> Option<ComponentIdx> {
        println!(
            "Checking {:?} {} children",
            src.name(),
            src.children().len()
        );
        for child_idx in src.children().iter().copied() {
            let child = &self.component_arena[child_idx];
            println!("Check {:?} -> {:?}", src.name(), child.name());
            if child.name() == Some(name) {
                return Some(child_idx);
            }
        }
        println!(
            "Parent {:?}",
            src.parent().map(|p| self.component_arena[p].name())
        );
        if let Some(parent) = src.parent() {
            self.find_component(&self.component_arena[parent], name)
        } else {
            // check root
            for child_idx in self.child_components.iter().copied() {
                let child = &self.component_arena[child_idx];
                println!("Check {:?} -> {:?}", src.name(), child.name());
                if child.name() == Some(name) {
                    return Some(child_idx);
                }
            }

            None
        }
    }

    fn with_addrmap<T>(&mut self, idx: ComponentIdx, f: impl FnOnce(&mut AddrMapType) -> T) -> T {
        if let AllComponent::AddrMap(addrmap) = &mut self.component_arena[idx] {
            f(addrmap)
        } else {
            panic!("Not an addrmap");
        }
    }

    fn add_addrmap(
        &mut self,
        parent: Option<ComponentIdx>,
        name: &str,
        body: &ComponentBody,
    ) -> Result<ComponentIdx, anyhow::Error> {
        println!("Adding addrmap {}", name);
        let addrmap = AddrMapType {
            parent,
            name: name.to_string(),
            ..Default::default()
        };
        self.component_arena.push(AllComponent::AddrMap(addrmap));
        let addrmap_idx = self.component_arena.len() - 1;
        for elem in body.elements.iter() {
            match elem {
                ComponentBodyElem::ComponentDef(component) => {
                    let comp = self.convert_component(parent.clone(), component)?;
                    if let Some(comp_idx) = comp {
                        let comp = &self.component_arena[comp_idx];
                        if component.insts.is_some() {
                            match comp {
                                AllComponent::Reg(_) => {
                                    let new_insts = self.convert_instances(
                                        comp_idx,
                                        component.insts.as_ref().unwrap(),
                                    )?;
                                    self.with_addrmap(addrmap_idx, |addrmap| {
                                        addrmap.instances.extend(new_insts);
                                    });
                                }
                                _ => {}
                            }
                        }
                        self.with_addrmap(addrmap_idx, |addrmap| {
                            addrmap.children.push(comp_idx);
                        });
                        // comp.clone().as_field().map(|f| {
                        //     if let Some(name) = &f.name {
                        //         println!("\nInserting field {} into map", name);
                        //         addrmap.fields.insert(name.clone(), Rc::new(f.clone()));
                        //     }
                        // });
                    }
                }
                ComponentBodyElem::EnumDef(enum_def) => {
                    let e = self.parse_enum(enum_def)?;
                    self.with_addrmap(addrmap_idx, |addrmap| {
                        addrmap.enums.push(e);
                    });
                }
                ComponentBodyElem::StructDef(_struct_def) => todo!(),
                ComponentBodyElem::ConstraintDef(_constraint_def) => todo!(),
                ComponentBodyElem::ExplicitComponentInst(explicit_component_inst) => {
                    println!("Explicit component inst: {:?}", explicit_component_inst);
                    if let Some(component_idx) = self.find_component(
                        &self.component_arena[addrmap_idx],
                        &explicit_component_inst.id,
                    ) {
                        println!(
                            "Found component: {:?}",
                            self.component_arena[component_idx].name()
                        );
                        todo!()
                    } else {
                        bail!(
                            "Component {} not found in scope",
                            explicit_component_inst.id
                        );
                    }
                }
                ComponentBodyElem::PropertyAssignment(property_assignment) => {
                    //println!("Property assignment: {:?}", property_assignment);
                    if let Some((key, value)) = self.evaluate_property(property_assignment) {
                        self.with_addrmap(addrmap_idx, |addrmap| {
                            addrmap.properties.insert(key, value);
                        });
                    }
                }
            }
        }
        //println!("Properties {}: {:?}", name, addrmap.properties);
        Ok(addrmap_idx)
    }

    fn convert_instances(
        &self,
        reg: ComponentIdx,
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
                Some(self.evaluate_constant_expr_int(&eq)?.value as usize)
            } else {
                None
            };
            // TODO: support regwidth
            let inst = RegisterInstance {
                name: inst.id.clone(),
                offset,
                width: 32,
                type_: reg,
            };
            instances.push(inst);
        }
        Ok(instances)
    }

    fn evaluate_property(
        &self,
        property_assignment: &PropertyAssignment,
    ) -> Option<(String, StringOrInt)> {
        match property_assignment {
            PropertyAssignment::ExplicitOrDefaultPropAssignment(pa) => match pa {
                ExplicitOrDefaultPropAssignment::ExplicitPropModifier(
                    _default_keyword,
                    _explicit_prop_modifier,
                ) => todo!(),
                ExplicitOrDefaultPropAssignment::ExplicitPropAssignment(_default, epa) => match epa
                {
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
                                PropAssignmentRhs::ConstantExpr(constant_expr) => self
                                    .evaluate_constant_expr_str(constant_expr)
                                    .ok()
                                    .map(StringOrInt::String)
                                    .or(self
                                        .evaluate_constant_expr_int(constant_expr)
                                        .map(StringOrInt::Int)
                                        .ok()),
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

    fn evaluate_constant_expr_str(&self, expr: &ConstantExpr) -> Result<String, anyhow::Error> {
        match expr {
            ConstantExpr::ConstantPrimary(prim, cont) => {
                if cont.is_some() {
                    bail!("Unsupported complex expression for string");
                }
                match prim {
                    ConstantPrimary::Base(constant_primary_base) => match constant_primary_base {
                        ConstantPrimaryBase::PrimaryLiteral(primary_literal) => {
                            match primary_literal {
                                PrimaryLiteral::StringLiteral(s) => Ok(s.clone()),
                                _ => bail!(
                                    "Unsupported literal in string evaluation context: {:?}",
                                    primary_literal
                                ),
                            }
                        }
                        _ => {
                            bail!("Unsupported expression for string");
                        }
                    },
                    ConstantPrimary::Cast(_constant_primary_base, _constant_expr) => {
                        bail!("Casting string not supported")
                    }
                }
            }
            ConstantExpr::UnaryOp(op, _expr, _cont) => {
                bail!("Unsupported unary operation on string: {:?}", op);
            }
        }
    }

    fn evaluate_constant_expr_cont_int(
        &self,
        val: Integer,
        cont: &Option<Box<ConstantExprContinue>>,
    ) -> Result<Integer, anyhow::Error> {
        match cont {
            None => Ok(val),
            Some(cont) => {
                match cont.as_ref() {
                    ConstantExprContinue::BinaryOp(op, expr, _cont) => {
                        let rhs = self.evaluate_constant_expr_int(expr.as_ref())?;

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
                        let b = self.evaluate_constant_expr_int(b.as_ref())?;
                        let c = self.evaluate_constant_expr_int(c.as_ref())?;
                        if a == TRUE {
                            self.evaluate_constant_expr_cont_int(b, cont)
                        } else {
                            self.evaluate_constant_expr_cont_int(c, cont)
                        }
                    }
                }
            }
        }
    }

    fn evaluate_primary_literal_int(&self, p: &PrimaryLiteral) -> Result<Integer, anyhow::Error> {
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
        &self,
        base: &ConstantPrimaryBase,
    ) -> Result<Integer, anyhow::Error> {
        match base {
            ConstantPrimaryBase::PrimaryLiteral(p) => self.evaluate_primary_literal_int(p),
            ConstantPrimaryBase::ConstantExpr(c) => self.evaluate_constant_expr_int(c),
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

    fn evaluate_cast(
        &self,
        _value: Integer,
        _expr: &ConstantExpr,
    ) -> Result<Integer, anyhow::Error> {
        bail!("Casting not supported");
    }

    fn evaluate_constant_primary_int(
        &self,
        prim: &ConstantPrimary,
    ) -> Result<Integer, anyhow::Error> {
        match prim {
            ConstantPrimary::Base(base) => self.evaluate_constant_primary_base_int(base),
            ConstantPrimary::Cast(base, cast) => {
                let base = self.evaluate_constant_primary_base_int(base)?;
                self.evaluate_cast(base, cast.as_ref())
            }
        }
    }

    fn evaluate_constant_expr_int(&self, expr: &ConstantExpr) -> Result<Integer, anyhow::Error> {
        match expr {
            ConstantExpr::ConstantPrimary(prim, cont) => {
                let val = self.evaluate_constant_primary_int(prim)?;
                self.evaluate_constant_expr_cont_int(val, cont)
            }
            ConstantExpr::UnaryOp(op, expr, cont) => {
                let expr = self.evaluate_constant_expr_int(expr)?;
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
                self.evaluate_constant_expr_cont_int(val, cont)
            }
        }
    }

    fn parse_enum(&self, e: &EnumDef) -> Result<Enum, anyhow::Error> {
        let mut values = vec![];
        let mut last_value: Option<Integer> = None;
        for entry in e.body.iter() {
            let val = match (&last_value, &entry.expr) {
                (None, None) => Integer {
                    width: 32,
                    value: 0,
                },
                (Some(last_val), None) => last_val.add(1),
                (_, Some(expr)) => self.evaluate_constant_expr_int(expr)?,
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

/// Component is the type of an instance.
trait Component {
    fn as_field(&self) -> Option<&FieldType> {
        None
    }
    fn name(&self) -> Option<&str>;
    fn component_type(&self) -> ComponentType;
    fn parent(&self) -> Option<ComponentIdx>;
    fn width(&self) -> usize;
    fn offset(&self) -> usize;
    fn fields(&self) -> &HashMap<String, ComponentIdx>;
    fn children(&self) -> &[ComponentIdx];
    fn enums(&self) -> &[Enum];
    fn properties(&self) -> &HashMap<String, StringOrInt>;
}

#[derive(Clone)]
struct RegisterInstance {
    name: String,
    offset: Option<usize>,
    width: usize,
    type_: ComponentIdx,
}

#[derive(Clone, Default)]
struct AddrMapType {
    name: String,
    offset: usize,
    width: usize,
    parent: Option<ComponentIdx>,
    children: Vec<ComponentIdx>,
    fields: HashMap<String, ComponentIdx>,
    instances: Vec<RegisterInstance>,
    enums: Vec<Enum>,
    properties: HashMap<String, StringOrInt>,
}

impl Component for AddrMapType {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
    fn component_type(&self) -> ComponentType {
        ComponentType::AddrMap
    }
    fn parent(&self) -> Option<ComponentIdx> {
        self.parent
    }
    fn fields(&self) -> &HashMap<String, ComponentIdx> {
        &self.fields
    }
    fn width(&self) -> usize {
        self.width
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn children(&self) -> &[ComponentIdx] {
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

    let _root_root = World::parse(&root)?;
    for d in root.descriptions.iter() {
        match d {
            Description::ComponentDef(c) => {
                let t = c.def.type_;
                let name = c.def.name.clone();
                if let Some(name) = name.as_deref() {
                    let body = &c.def.body;
                    if t == ComponentType::AddrMap && addrmaps.contains(&name) {
                        println!("Component {:?} {}", t, name);
                        enumerate_instances(&root, body);
                    }
                }
            }
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
