use std::marker::PhantomData;
use std::rc::Rc;

use crate::{environment::{Environment, features::{FeatureBuilder, arrays::ArraysFeatureBuilder, debug::DebugFeatureBuilder, files::FilesFeatureBuilder, math::MathFeatureBuilder, numbers::NumbersFeatureBuilder, strings::StringsFeatureBuilder}}, error::RuntimeError, module::RuntimeModule};
use otr_core::error::Result;

pub(crate) trait EnvironmentBuilderState { }

pub struct EnvironmentBuilderBaseState;
impl EnvironmentBuilderState for EnvironmentBuilderBaseState { }

pub struct EnvironmentBuilderFeatureState;
impl EnvironmentBuilderState for EnvironmentBuilderFeatureState { }

#[allow(private_bounds)]
pub struct EnvironmentBuilder<State: EnvironmentBuilderState> {
    features: Vec<(String, RuntimeModule<'static>)>,
    current_feature: Option<(String, Box<dyn FeatureBuilder>)>,

    phantom_data: PhantomData<State>,
}

#[allow(private_bounds, private_interfaces)]
impl EnvironmentBuilder<EnvironmentBuilderBaseState> {
    #[allow(private_bounds)]
    pub fn new() -> EnvironmentBuilder<EnvironmentBuilderBaseState> {
        EnvironmentBuilder {
            features: Vec::new(),
            current_feature: None,

            phantom_data: PhantomData,
        }
    }
}

impl Default for EnvironmentBuilder<EnvironmentBuilderBaseState> {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentBuilder<EnvironmentBuilderBaseState> {
    pub fn with_feature(self, feature_ident: impl AsRef<str>) -> Result<EnvironmentBuilder<EnvironmentBuilderFeatureState>> {
        let feature_builder = match feature_ident.as_ref() as &str {

            "Arrays" => ArraysFeatureBuilder::new_boxed(),
            "Debug" => DebugFeatureBuilder::new_boxed(),
            "Files" => FilesFeatureBuilder::new_boxed(),
            "Numbers" => NumbersFeatureBuilder::new_boxed(),
            "Strings" => StringsFeatureBuilder::new_boxed(),
            "Math" => MathFeatureBuilder::new_boxed(),

            other => return Err(RuntimeError::Unknown {
                message: format!("Unknown feature: {}", other)
            }.boxed())
        };

        Ok(EnvironmentBuilder {
            features: self.features,
            current_feature: Some((feature_ident.as_ref().to_string(), feature_builder)),

            phantom_data: PhantomData,
        })
    }

    pub fn build(self) -> Environment<'static> {
        let mut environment = Environment::new(String::new(), 0);

        for (module_identifier, module) in self.features {
            environment.load_module(module_identifier, Rc::new(module));
        }

        environment
    }
}

impl EnvironmentBuilder<EnvironmentBuilderFeatureState> {
    pub fn with_arg(mut self, arg_ident: impl AsRef<str>, arg_value: impl AsRef<str>) -> Result<EnvironmentBuilder<EnvironmentBuilderFeatureState>> {
        self.current_feature.as_mut().unwrap().1.add_arg(&arg_ident.as_ref(), &arg_value.as_ref())?;

        Ok(self)
    }

    pub fn finalize_feature(mut self) -> Result<EnvironmentBuilder<EnvironmentBuilderBaseState>> {
        let (module_identifier, mut module);

        if let Some(current_feature) = self.current_feature {
            (module_identifier, module) = current_feature;
        } else {
            unreachable!()
        }

        let module = module.build()?;

        self.features.push((module_identifier, module));

        Ok(EnvironmentBuilder {
            features: self.features,
            current_feature: None,

            phantom_data: PhantomData
        })
    }
}