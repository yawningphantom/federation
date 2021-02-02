//! A `Request` for a spec within a document.
//!
//! `Request`s are derived from `Directive`s during schema bootstrapping.

import { customScalar, Deserialized, err, Must, obj, ok, str } from '../metadata'
import { spec, Spec } from '../spec'

export const core = spec `https://lib.apollo.dev/core/v0.1`

export const SpecUrl = customScalar(
  (repr: string) => {
    try {
      return ok(Spec.parse(repr))
    } catch(error) {
      return err(error)
    }
  }
)

export const Using = obj({
  using: SpecUrl.must,
  as: str,
})

export type Using = Must<Deserialized<typeof Using>>
