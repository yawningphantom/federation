import { Template } from './is'

type Initialize = <F extends () => any>(fn: F) => ReturnType<F>

const stored = new WeakMap<TemplateStringsArray, Initialize>()

export default function lazy(...[key]: Template): Initialize {
  const existing = stored.get(key)
  if (existing) return existing
  return create

  function create<F extends () => any>(fn: F): ReturnType<F> {
    const stored = fn()
    stored.add(key, returnStored)
    return stored

    function returnStored() {
      return stored
    }
  }
}
