use rustc_hir::def_id::{DefIndex, LocalDefId};
use rustc_hir::{HirId, ItemLocalId};

struct HirIdDef {
    pub owner: LocalDefId,
    pub local_id: ItemLocalId,
}

struct LocalDefIdDef {
    pub local_def_index: DefIndex,
}

struct DefIndexDef {

}