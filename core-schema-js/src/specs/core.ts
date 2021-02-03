//! A `Request` for a spec within a document.
//!
//! `Request`s are derived from `Directive`s during schema bootstrapping.

import { customScalar, Deserialized, Must, Str, Bool } from '../metadata'
import { spec, Spec } from '../spec'
import data from '../data'
import { DirectiveLocation } from 'graphql'
import layer from '../layer'

export const core = spec `https://lib.apollo.dev/core/v0.1`

export const SpecUrl = customScalar(Spec)

export const Using =
  core.input `Using` ({
    using: SpecUrl.must,
    as: Str,
  }, 'repeatable on', 'SCHEMA')

export const name = data <String, Using> `Name for spec within document`
  .orElse((using: Using) => using.as ?? using.using.name)

export const Export =
  core.input `Export` ({
    export: Bool.must
  }, 'on', ...Object.values(DirectiveLocation))

export type Using = Must<Deserialized<typeof Using>>
export type Export = Must<Deserialized<typeof Export>>

export default layer(Using, Export)
