import { mat3, vec2 } from 'gl-matrix';

export class Transform {
  private _position: vec2;
  private _pivot: vec2;
  private _scale: vec2;
  private _rotation: number;
  private _alpha: number;
  private _isDirty: boolean;

  constructor() {
    this._position = vec2.create();
    this._pivot = vec2.create();
    this._scale = vec2.create();
    this._rotation = 0;
    this._alpha = 1;
    this._isDirty = true;
  }

  public get position(): vec2 {
    return this._position;
  }

  public set position(value: vec2) {
    this._position = value;
    this._isDirty = true;
  }

  public get pivot(): vec2 {
    return this._pivot;
  }

  public set pivot(value: vec2) {
    this._pivot = value;
    this._isDirty = true;
  }

  public get scale(): vec2 {
    return this._scale;
  }

  public set scale(value: vec2) {
    this._scale = value;
    this._isDirty = true;
  }

  public get rotation(): number {
    return this._rotation;
  }

  public set rotation(value: number) {
    this._rotation = value;
    this._isDirty = true;
  }

  public get alpha(): number {
    return this._alpha;
  }

  public set alpha(value: number) {
    this._alpha = value;
    this._isDirty = true;
  }

  public isDirty(): boolean {
    return this._isDirty;
  }

  public clearDirty() {
    this._isDirty = false;
  }
}

export class GlobalTransform {
  private _matrix: mat3;
  private _alpha: number;
  private _isDirty: boolean;

  constructor() {
    this._matrix = mat3.create();
    this._alpha = 1;
    this._isDirty = false;
  }

  // 성능 최적화용: 재사용할 임시 객체
  private _t1 = mat3.create();
  private _r = mat3.create();
  private _s = mat3.create();
  private _t2 = mat3.create();
  private _pivot_transform = mat3.create();
  private _translation = mat3.create();
  private _offset = vec2.create();

  public update(parent: GlobalTransform, local: Transform) {
    if (!parent._isDirty && !local.isDirty()) return;

    const t1 = this._t1;
    const r = this._r;
    const s = this._s;
    const t2 = this._t2;
    const pivotTransform = this._pivot_transform;
    const translation = this._translation;
    const offset = this._offset;

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

    mat3.multiply(this._matrix, parent._matrix, localMatrix);
    mat3.multiply(this._matrix, this._matrix, pivotTransform);

    this._alpha = parent._alpha * local.alpha;
    this._isDirty = true;

    local.clearDirty(); // optional setter
  }

  public finalize() {
    this._isDirty = false;
  }

  // 아래와 같은 함수는 매 프레임마다 오브젝트를 생성시키므로, 성능에 좋지 않아 사용하지 않음
  /*public extract() {
    this.finalize();
    return { matrix: this._matrix, alpha: this._alpha };
  }*/
}
