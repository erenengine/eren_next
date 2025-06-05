import { mat3, vec2 } from 'gl-matrix';

export class LocalTransform {
  #position: vec2;
  #pivot: vec2;
  #scale: vec2;
  #rotation: number;
  #alpha: number;
  #isDirty: boolean;

  constructor() {
    this.#position = vec2.create();
    this.#pivot = vec2.create();
    this.#scale = vec2.create();
    this.#rotation = 0;
    this.#alpha = 1;
    this.#isDirty = true;
  }

  public get position(): vec2 {
    return this.#position;
  }

  public set position(value: vec2) {
    this.#position = value;
    this.#isDirty = true;
  }

  public get pivot(): vec2 {
    return this.#pivot;
  }

  public set pivot(value: vec2) {
    this.#pivot = value;
    this.#isDirty = true;
  }

  public get scale(): vec2 {
    return this.#scale;
  }

  public set scale(value: vec2) {
    this.#scale = value;
    this.#isDirty = true;
  }

  public get rotation(): number {
    return this.#rotation;
  }

  public set rotation(value: number) {
    this.#rotation = value;
    this.#isDirty = true;
  }

  public get alpha(): number {
    return this.#alpha;
  }

  public set alpha(value: number) {
    this.#alpha = value;
    this.#isDirty = true;
  }

  public isDirty(): boolean {
    return this.#isDirty;
  }

  public clearDirty() {
    this.#isDirty = false;
  }
}

export class GlobalTransform {
  #matrix: mat3;
  #alpha: number;
  #isDirty: boolean;

  constructor() {
    this.#matrix = mat3.create();
    this.#alpha = 1;
    this.#isDirty = false;
  }

  // 성능 최적화용: 재사용할 임시 객체
  #t1 = mat3.create();
  #r = mat3.create();
  #s = mat3.create();
  #t2 = mat3.create();
  #pivot_transform = mat3.create();
  #translation = mat3.create();
  #offset = vec2.create();

  public update(parent: GlobalTransform, local: LocalTransform) {
    if (!parent.#isDirty && !local.isDirty()) return;

    const t1 = this.#t1;
    const r = this.#r;
    const s = this.#s;
    const t2 = this.#t2;
    const pivotTransform = this.#pivot_transform;
    const translation = this.#translation;
    const offset = this.#offset;

    // pivot_transform = T(pivot) * R(rotation) * S(scale) * T(-pivot)
    mat3.fromTranslation(t1, local.pivot);
    mat3.fromRotation(r, local.rotation);
    mat3.fromScaling(s, local.scale);

    vec2.negate(offset, local.pivot);
    mat3.fromTranslation(t2, offset);

    mat3.multiply(pivotTransform, t1, r);
    mat3.multiply(pivotTransform, pivotTransform, s);
    mat3.multiply(pivotTransform, pivotTransform, t2);

    // local_matrix = T(position - pivot) * pivot_transform
    vec2.sub(offset, local.position, local.pivot);
    mat3.fromTranslation(translation, offset);
    const localMatrix = translation;

    mat3.multiply(this.#matrix, parent.#matrix, localMatrix);
    mat3.multiply(this.#matrix, this.#matrix, pivotTransform);

    this.#alpha = parent.#alpha * local.alpha;
    this.#isDirty = true;

    local.clearDirty(); // optional setter
  }

  public finalize() {
    this.#isDirty = false;
  }

  // 아래와 같은 함수는 매 프레임마다 오브젝트를 생성시키므로, 성능에 좋지 않아 사용하지 않음
  /*public extract() {
    this.finalize();
    return { matrix: this.#matrix, alpha: this.#alpha };
  }*/

  public get matrix(): mat3 {
    return this.#matrix;
  }

  public get alpha(): number {
    return this.#alpha;
  }
}
