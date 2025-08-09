use ahash::AHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct WgslSource {
    pub name: &'static str,
    pub code: &'static str,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Topology { TriangleList, TriangleStrip, LineList }

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RenderState<TFmt: Hash + Eq + Copy> {
    pub format: TFmt,
    pub depth: bool,
    pub msaa: u32,
    pub topo: Topology,
}

#[derive(Clone, Default)]
pub struct Overrides {
    pub map: HashMap<String, f64>,
}

impl Overrides {
    pub fn with(mut self, name: &str, value: f64) -> Self {
        self.map.insert(name.to_string(), value);
        self
    }
    pub fn set_bool(&mut self, name: &str, v: bool) { self.map.insert(name.to_string(), if v { 1.0 } else { 0.0 }); }
    pub fn set_f32(&mut self,  name: &str, v: f32)  { self.map.insert(name.to_string(), v as f64); }
    pub fn get_map(&self) -> &HashMap<String, f64> { &self.map }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ShaderKey<TFmt: Hash + Eq + Copy> {
    pub src_name: &'static str,
    pub state: RenderState<TFmt>,
    pub consts_hash: u64,
}

impl<TFmt: Hash + Eq + Copy> ShaderKey<TFmt> {
    pub fn new(src: &WgslSource, state: RenderState<TFmt>, ov: &Overrides) -> Self {
        let mut h = AHasher::default();
        let mut pairs: Vec<_> = ov.map.iter().collect();
        pairs.sort_by(|a,b| a.0.cmp(b.0));
        for (k, v) in pairs {
            k.hash(&mut h);
            v.to_bits().hash(&mut h);
        }
        Self { src_name: src.name, state, consts_hash: h.finish() }
    }
}
