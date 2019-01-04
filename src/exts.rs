use std::slice::SliceIndex;

/// Trait to allow debug-assert slice indexing.
pub(crate) trait GetDebug {
  /// Index into a slice; this should use normal indexing in debug mode, but unchecked
  /// indexing in release mode.
  unsafe fn get_debug_checked<I: SliceIndex<Self>>(
    self: &Self,
    i: I,
  ) -> &<I as SliceIndex<Self>>::Output;
}

impl GetDebug for str {
  unsafe fn get_debug_checked<I: SliceIndex<Self>>(
    self: &Self,
    i: I,
  ) -> &<I as SliceIndex<Self>>::Output {
    if cfg!(debug_assertions) {
      self.get(i).unwrap()
    } else {
      self.get_unchecked(i)
    }
  }
}
