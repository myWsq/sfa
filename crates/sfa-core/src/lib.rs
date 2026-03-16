pub mod archive;
pub mod codec;
pub mod config;
pub mod error;
pub mod format;
pub mod integrity;
pub mod model;
pub mod planner;
pub mod stats;

pub use archive::{ArchiveReader, PreparedArchive, prepare_archive, write_archive};
pub use config::{
    DataCodec, FrameHashAlgo, IntegrityMode, ManifestCodec, ManifestHashAlgo, OverwritePolicy,
    PackConfig, RestoreOwnerPolicy, UnpackConfig,
};
pub use error::{Error, Result};
pub use format::{
    FRAME_HEADER_LEN, FeatureFlags, FrameHeaderV1, HEADER_LEN, HeaderV1, TRAILER_LEN, TrailerV1,
};
pub use model::{
    BundleInput, BundleKind, BundlePart, BundlePlanRecord, EncodedFrame, EntryKind, EntryRecord,
    ExtentRecord, Manifest, PlannerInputEntry,
};
pub use planner::{PlannedArchive, plan_archive};
pub use stats::{
    ObservationStatus, ObservedMetric, PackPhaseBreakdown, PackStats, UnpackPhaseBreakdown,
    UnpackStats,
};
