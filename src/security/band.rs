use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InformationBand {
    Sealed,
    Local,
    Session,
    Control,
    Public,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryTarget {
    None,
    LocalMesh,
    LoopbackBrowser,
    ExternalControl,
    PublicNetwork,
}

/// Declassification is never implicit; each band has one reviewed boundary.
pub fn band_allows_target(band: InformationBand, target: BoundaryTarget) -> bool {
    matches!(
        (band, target),
        (InformationBand::Sealed, BoundaryTarget::None)
            | (InformationBand::Local, BoundaryTarget::LocalMesh)
            | (InformationBand::Session, BoundaryTarget::LoopbackBrowser)
            | (InformationBand::Control, BoundaryTarget::ExternalControl)
            | (InformationBand::Public, BoundaryTarget::PublicNetwork)
    )
}

pub(crate) fn browser_visible(band: InformationBand) -> bool {
    matches!(band, InformationBand::Session | InformationBand::Public)
}
