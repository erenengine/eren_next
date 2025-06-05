export class AssetsState<A> {
  ready: Set<A>;
  pending: Map<A, string>;

  constructor() {
    this.ready = new Set();
    this.pending = new Map();
  }
}
