import { asString, AsString, Template } from "./is"

/**
 * A Datum<D, T> describes some arbitrary data D attached to some type T, which
 * must be an object (default: object).
 */
export interface Data<D, T=any> {
  /**
   * Access the data from `target`
   */
  (target: T): D
}

export interface MaybeData<D, T=any> {
  /**
   * Access the data from `target`
   */
  (target: T): D | undefined

  /**
   * Returns a new accessor for the same underlying data. The returned accessor
   * will call the provided `setWith` with `target` if the datum does not
   * already exist on `target`, save the result on `target`, and return it.
   */
  orElse(setWith: (target: T) => D): Data<D, T>
}

export type DataValue<D extends Data<any>> = D extends Data<infer V> ? V : never

export type Description = Template | [string]

const internal = {
  write: Symbol('write')
}

export default data

export function data<D, T=any>(...description: AsString): MaybeData<D, T> {
  const name = asString(description)
  const symbol = Symbol(name)
  return Object.assign(get, {
    set,
    orElse,
    [internal.write]: write,
  })

  function get(target: T): D {
    return (target as any)[symbol]
  }

  function write(target: T, data: D) {
    (target as any)[symbol] = data
  }

  function orElse(setWith: (target: T) => D) {
    return Object.assign(getWithDefault, {
      set,
      [internal.write]: write,
    })

    function getWithDefault(target: T) {
      if (symbol in target) return get(target)
      const value = setWith(target)
      write(target, value)
      return value
    }
  }
}


export function set<T extends object, C extends Data<any, T>>(
  target: T,
  column: C,
  value: DataValue<C>): T
{
  (column as any)[internal.write](target, value)
  return target
  // return {
  //   mut: 'set' as 'set',
  //   target,
  //   data: column,
  //   value
  // }
}

export interface SetValue<T extends object, D=any> {
  mut: 'set'
  target: T,
  data: Data<D, T>
  value: D
}

export function update<T extends object>(target: T) {
  return function apply<U extends T>(...updates: SetValue<U>[]) {
    for (const { data, value } of updates) {
      (data as any)[internal.write](target, value)
    }
    return target
  }
}
