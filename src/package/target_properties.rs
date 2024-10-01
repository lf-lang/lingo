use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::path::{Path, PathBuf};

pub trait CMakeLoader {
    fn read_file(&mut self, path: &str) -> anyhow::Result<AutoCmakeLoad>;
}

impl Serialize for AutoCmakeLoad {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl Display for AutoCmakeLoad {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GenericTargetPropertiesFile {
    Library(LibraryTargetPropertiesFile),
    App(AppTargetPropertiesFile),
}

#[derive(Clone)]
pub enum GenericTargetProperties {
    Library(LibraryTargetProperties),
    App(AppTargetProperties),
}

#[derive(Clone, Default)]
pub struct AutoCmakeLoad(String);

impl MergeTargetProperty for AutoCmakeLoad {
    fn merge(&mut self, parent: &AutoCmakeLoad) -> anyhow::Result<()> {
        self.0 += &*("\n# ------------------------- \n".to_owned() + &*parent.0);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LibraryTargetPropertiesFile {
    /// cmake include only available for C and CPP
    #[serde(rename = "cmake-include", default)]
    cmake_include: Option<PathBuf>,

    /// files that should be compiled and linked
    #[serde(rename = "sources", default)]
    sources: Vec<PathBuf>,

    /// list of files that should be made available to the user
    #[serde(rename = "sources", default)]
    artifacts: Vec<PathBuf>,
}

#[derive(Clone, Default)]
pub struct LibraryTargetProperties {
    /// cmake include only available for C and CPP
    pub cmake_include: AutoCmakeLoad,

    /// files that should be compiled and linked
    pub sources: Vec<PathBuf>,

    /// list of files that should be made available to the user
    pub artifacts: Vec<PathBuf>,
}

impl LibraryTargetPropertiesFile {
    pub fn from(self, base_path: &Path) -> LibraryTargetProperties {
        LibraryTargetProperties {
            cmake_include: AutoCmakeLoad(
                self.cmake_include
                    .map(|cmake_file| {
                        let absolute_path = base_path.join(cmake_file);
                        std::fs::read_to_string(absolute_path)
                            .expect("invalid file {absolute_path}")
                    })
                    .unwrap_or_default(),
            ),
            sources: self.sources,
            artifacts: self.artifacts,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppTargetPropertiesFile {
    /// cmake include only available for C and CPP
    #[serde(rename = "cmake-include", default)]
    cmake_include: Option<PathBuf>,

    /// if the runtime should wait for physical time to catch up
    #[serde(default)]
    pub fast: bool,
}

#[derive(Clone, Default)]
pub struct AppTargetProperties {
    /// cmake include only available for C and CPP
    cmake_include: AutoCmakeLoad,

    /// if the runtime should wait for physical time to catch up
    pub fast: bool,
}

impl AppTargetPropertiesFile {
    pub fn from(self, base_path: &Path) -> AppTargetProperties {
        AppTargetProperties {
            cmake_include: AutoCmakeLoad(
                self.cmake_include
                    .map(|cmake_file| {
                        let absolute_path = base_path.join(cmake_file);
                        std::fs::read_to_string(&absolute_path)
                            .expect("invalid file {absolute_path}")
                    })
                    .unwrap_or_default(),
            ),
            fast: self.fast,
        }
    }
}

pub trait MergeTargetProperty {
    fn merge(&mut self, other: &Self) -> anyhow::Result<()>;
}

pub trait MergeTargetProperties {
    fn merge(&mut self, other: &LibraryTargetProperties) -> anyhow::Result<()>;
}

impl MergeTargetProperties for LibraryTargetProperties {
    fn merge(&mut self, partent: &LibraryTargetProperties) -> anyhow::Result<()> {
        self.cmake_include.merge(&partent.cmake_include)?;
        Ok(())
    }
}

impl MergeTargetProperties for AppTargetProperties {
    fn merge(&mut self, parent: &LibraryTargetProperties) -> anyhow::Result<()> {
        self.cmake_include.merge(&parent.cmake_include)?;
        Ok(())
    }
}

impl MergeTargetProperties for GenericTargetProperties {
    fn merge(&mut self, other: &LibraryTargetProperties) -> anyhow::Result<()> {
        match self {
            GenericTargetProperties::Library(own_lib) => own_lib.merge(other),
            GenericTargetProperties::App(own_app) => own_app.merge(other),
        }
    }
}

impl AppTargetProperties {
    pub fn write_artifacts(&self, library_folder: &Path) -> anyhow::Result<()> {
        let file = library_folder.join("aggregated_cmake_include.cmake");

        let mut fd = std::fs::File::create(file)?;
        fd.write_all(self.cmake_include.0.as_ref())?;
        fd.flush()?;

        Ok(())
    }
}
