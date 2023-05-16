// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    rc::Rc,
};

use derivative::Derivative;

use crate::core::{Expression, ModuleLoader};

impl<'a, TLoader: ModuleLoader> ModuleLoader for &'a TLoader {
    type Output = TLoader::Output;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        let loader = self.deref();
        loader.load(import_path, current_path)
    }
}

#[derive(Derivative)]
#[derivative(Default(bound = ""), Clone(bound = ""), Debug(bound = ""))]
pub struct NoopModuleLoader<T: Expression> {
    _expression: PhantomData<T>,
}

impl<T: Expression> ModuleLoader for NoopModuleLoader<T> {
    type Output = T;
    fn load(
        &self,
        _import_path: &str,
        _current_path: &Path,
    ) -> Option<Result<Self::Output, String>> {
        None
    }
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = "T1: Clone, T2: Clone"),
    Debug(bound = "T1: std::fmt::Debug, T2: std::fmt::Debug")
)]
pub struct ChainedModuleLoader<
    T: Expression,
    T1: ModuleLoader<Output = T>,
    T2: ModuleLoader<Output = T>,
> {
    left: T1,
    right: T2,
    _value: PhantomData<T>,
}

impl<T: Expression, T1: ModuleLoader<Output = T>, T2: ModuleLoader<Output = T>>
    ChainedModuleLoader<T, T1, T2>
{
    pub fn new(left: T1, right: T2) -> Self {
        Self {
            left,
            right,
            _value: PhantomData,
        }
    }
}

impl<T: Expression, T1: ModuleLoader<Output = T>, T2: ModuleLoader<Output = T>> ModuleLoader
    for ChainedModuleLoader<T, T1, T2>
{
    type Output = T;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        match self.left.load(import_path, current_path) {
            Some(result) => Some(result),
            None => self.right.load(import_path, current_path),
        }
    }
}

#[derive(Derivative)]
#[derivative(
    Default(bound = ""),
    Clone(bound = "TLoader: Clone"),
    Debug(bound = "TLoader: std::fmt::Debug")
)]
pub struct MaybeModuleLoader<TLoader: ModuleLoader> {
    inner: Option<TLoader>,
}

impl<TLoader: ModuleLoader> MaybeModuleLoader<TLoader> {
    pub fn new(inner: Option<TLoader>) -> Self {
        Self { inner }
    }
    pub fn take(&mut self) -> Option<TLoader> {
        self.inner.take()
    }
    pub fn replace(&mut self, value: TLoader) -> Option<TLoader> {
        self.inner.replace(value)
    }
}

impl<TLoader: ModuleLoader> ModuleLoader for MaybeModuleLoader<TLoader> {
    type Output = TLoader::Output;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        match self.inner.as_ref() {
            None => None,
            Some(loader) => loader.load(import_path, current_path),
        }
    }
}

pub struct BoxedModuleLoader<T: Expression> {
    inner: Box<dyn ModuleLoader<Output = T>>,
    _value: PhantomData<T>,
}

impl<T: Expression> BoxedModuleLoader<T> {
    pub fn new(loader: impl ModuleLoader<Output = T> + 'static) -> Self {
        Self {
            inner: Box::new(loader),
            _value: PhantomData,
        }
    }
}

impl<T: Expression> Deref for BoxedModuleLoader<T> {
    type Target = dyn ModuleLoader<Output = T>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: Expression> DerefMut for BoxedModuleLoader<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<T: Expression> ModuleLoader for BoxedModuleLoader<T> {
    type Output = T;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        let loader = self.inner.deref();
        loader.load(import_path, current_path)
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct RecursiveModuleLoader<T: Expression> {
    inner: Rc<RefCell<MaybeModuleLoader<BoxedModuleLoader<T>>>>,
}

impl<T: Expression> RecursiveModuleLoader<T> {
    pub fn new<TLoader: ModuleLoader<Output = T>>(
        factory: impl FnOnce(BoxedModuleLoader<T>) -> TLoader,
    ) -> Self
    where
        T: 'static,
        TLoader: 'static,
    {
        let cell = Rc::new(RefCell::new(MaybeModuleLoader::default()));
        let loader = factory(BoxedModuleLoader::new(RecursiveModuleLoader {
            inner: Rc::clone(&cell),
        }));
        {
            let mut inner = cell.deref().borrow_mut();
            inner.deref_mut().replace(BoxedModuleLoader::new(loader));
        }
        Self { inner: cell }
    }
}

impl<T: Expression> ModuleLoader for RecursiveModuleLoader<T> {
    type Output = T;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        let inner = self.inner.borrow();
        let loader = inner.deref();
        loader.load(import_path, current_path)
    }
}

#[derive(Clone, Debug)]
pub struct StaticModuleLoader<T: Expression> {
    modules: HashMap<String, T>,
}

impl<T: Expression> StaticModuleLoader<T> {
    pub fn new(modules: impl IntoIterator<Item = (String, T)>) -> Self {
        Self {
            modules: modules.into_iter().collect(),
        }
    }
}

impl<T: Expression> ModuleLoader for StaticModuleLoader<T> {
    type Output = T;
    fn load(
        &self,
        import_path: &str,
        _current_path: &Path,
    ) -> Option<Result<Self::Output, String>> {
        self.modules.get(import_path).cloned().map(Ok)
    }
}

#[derive(Derivative)]
#[derivative(
    Default(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = "")
)]
pub struct ErrorFallbackModuleLoader<T> {
    _value: PhantomData<T>,
}

impl<T: Expression> ModuleLoader for ErrorFallbackModuleLoader<T> {
    type Output = T;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<T, String>> {
        Some(Err(
            match get_module_path_metadata(import_path, current_path) {
                Ok(Some(metadata)) => match metadata.is_dir() {
                    true => String::from("Module path is a directory"),
                    false => String::from("No compatible loaders"),
                },
                Ok(None) => String::from("Module not found"),
                Err(error) => error,
            },
        ))
    }
}

fn get_module_path_metadata(
    import_path: &str,
    module_path: &Path,
) -> Result<Option<std::fs::Metadata>, String> {
    match std::fs::metadata(get_module_filesystem_path(import_path, module_path)) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => Ok(None),
            _ => Err(format!("{}", error)),
        },
    }
}

pub fn get_module_filesystem_path(import_path: &str, module_path: &Path) -> PathBuf {
    module_path
        .parent()
        .map(|parent| parent.join(import_path))
        .unwrap_or_else(|| Path::new(import_path).to_path_buf())
}
