use std::marker::PhantomData;
use std::rc::Rc;

use crate::runtime::error::RuntimeError;
use crate::runtime::environment::Environment;
use crate::runtime::environment::features::arrays::ArraysFeatureBuilder;
use crate::runtime::environment::features::debug::DebugFeatureBuilder;
use crate::runtime::environment::features::files::FilesFeatureBuilder;
use crate::runtime::environment::features::numbers::NumbersFeatureBuilder;
use crate::runtime::environment::features::strings::StringsFeatureBuilder;
use crate::runtime::environment::features::FeatureBuilder;
use crate::runtime::module::RuntimeModule;
use crate::error::Result;

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

impl EnvironmentBuilder<EnvironmentBuilderBaseState> {
    pub fn with_feature(self, feature_ident: impl AsRef<str>) -> Result<EnvironmentBuilder<EnvironmentBuilderFeatureState>> {
        let feature_builder = match feature_ident.as_ref() as &str {

            "Arrays" => ArraysFeatureBuilder::new(),
            "Debug" => DebugFeatureBuilder::new(),
            "Files" => FilesFeatureBuilder::new(),
            "Numbers" => NumbersFeatureBuilder::new(),
            "Strings" => StringsFeatureBuilder::new(),

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
        let mut environment = Environment::new(String::new());

        for (module_identifier, module) in self.features {
            environment.load_module(module_identifier, Rc::new(module));
        }

        todo!()
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