//! Layer 1 of the metadata pipeline: application-level specifications.
//!
//! This module declares the source model used to describe entities and their
//! relationships before resolution:
//! - [`crate::spec::entity_spec`] defines entities, fields, facets, and presentation hints.
//! - [`crate::spec::relationship`] defines references and compositions between entity sets.
//! - [`crate::spec::meta_package`] provides the built-in self-hosting meta model.
//! - [`crate::spec::synth_records`] generates synthetic admin/config records from specs.
pub mod entity_spec;
pub mod meta_package;
pub mod relationship;
pub mod synth_records;

pub mod package {
  #[allow(dead_code)]
    pub struct Package {
        name: String,
    }

    impl Package {
        pub fn new(name: &str) -> Self {
            Self { name: name.into() }
        }
    }
}

pub mod app {
    use serde_json::{json, Value};

    use crate::spec::package::Package;

  #[allow(dead_code)]
    pub struct App {
        name: String,
        title: String,
        main_entity: String,
        subtitle: Option<String>,
        icon: Option<String>,
        package: Package,
    }

    impl App {
        pub fn new(
            name: String,
            title: String,
            main_entity: String,
            subtitle: Option<String>,
            icon: Option<String>,
            package: Package,
        ) -> Self {
            Self {
                name,
                title,
                main_entity,
                subtitle,
                icon,
                package,
            }
        }

        pub fn apps_json_entry(&self) -> Value {
            json!({
                  "sap.app": {
                    "crossNavigation": {
                      "inbounds": {
                        format!("{}-display", self.main_entity): {
                          "action": "display",
                          "semanticObject": self.main_entity,
                          "signature": {
                            "additionalParameters": "allowed",
                            "parameters": {}
                          }
                        }
                      }
                    },
                    "id": self.name,
                    "title": self.title,
                    "subTitle": self.subtitle
                  },
                  "sap.flp": {
                    "type": "application"
                  },
                  "sap.platform.runtime": {
                    "componentProperties": {
                      "url": format!("./apps/{}/", self.main_entity)
                    }
                  },
                  "sap.ui": {
                    "deviceTypes": {
                      "desktop": true,
                      "phone": true,
                      "tablet": true
                    },
                    "technology": "UI5"
                  },
                  "sap.ui5": {
                    "componentName": self.name
                  }
                }
            )
        }

        #[allow(dead_code)]
        pub(crate) fn set_name(&self) -> &str {
            todo!()
        }
    }
}
pub use entity_spec::*;
pub use relationship::*;

// Legacy re-exports — existing code uses these from crate::annotations::*
pub use crate::annotations::{
    AnnotationsDef, DataPointDef, FacetSectionDef, FieldDef, HeaderFacetDef, HeaderInfoDef,
    NavigationPropertyDef, TableFacetDef, ValueListDef,
};
