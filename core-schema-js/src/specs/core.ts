//! A `Request` for a spec within a document.
//!
//! `Request`s are derived from `Directive`s during schema bootstrapping.

import { customScalar, Deserialized, Must, obj, str } from '../metadata'
import { asResultFn } from '../err'
import { spec, Spec } from '../spec'

export const core = spec `https://lib.apollo.dev/core/v0.1`

export const SpecUrl = customScalar(asResultFn(Spec.parse))

export const Using = obj({
  using: SpecUrl.must,
  as: str,
})

export type Using = Must<Deserialized<typeof Using>>
