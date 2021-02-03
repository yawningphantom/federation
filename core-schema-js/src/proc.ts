import { Data, DataValue, set } from './data'

export interface Selection<D> {
  readonly only: D
  pipe<F extends (sel: this) => any>(fn: F): ReturnType<F>
  get<N extends Data<any, D>>(column: N): Selection<DataValue<N>>
}

export interface SelectionMut<D> extends Selection<D> {
  update(change: (data: D) => D): this
}

export function select<T extends object, C extends Data<any, T>>(
  target: T,
  column: C
): Selection<DataValue<C>> {
  return new SelFrom(target, column)
}

export abstract class Sel {
  get only() { return this }

  pipe<F extends (sel: this) => any>(fn: F): ReturnType<F> {
    return fn(this)
  }

  get<N extends Data<any, this["only"]>>(column: N): Selection<DataValue<N>> {
    return new SelFrom(this.only, column) as any
  }
}

export class SelFrom<T extends object, C extends Data<any, T>> extends Sel implements SelectionMut<DataValue<C>> {
  constructor(readonly target: T, readonly column: C) { super() }

  get only() {
    return this.column(this.target)
  }

  update(change: (data: DataValue<C>) => DataValue<C>) {
    set(this.target, this.column, change(this.column(this.target)))
    return this
  }
}
