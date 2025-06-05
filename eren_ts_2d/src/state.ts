import { mat3 } from 'gl-matrix';
import { AssetsState, InputState } from '@erenengine/next_core';

export interface RenderRequest<SA> {
  matrix: mat3;
  alpha: number;
  spriteAssetId: SA;
}

export class GameState<SA> {
  deltaTime: number;
  spriteAssets: AssetsState<SA>;
  input: InputState;
  renderRequests: RenderRequest<SA>[];
  windowSize: { width: number; height: number; };

  constructor() {
    this.deltaTime = 0;
    this.spriteAssets = new AssetsState<SA>();
    this.input = new InputState();
    this.renderRequests = [];
    this.windowSize = { width: 0, height: 0 };
  }
}
