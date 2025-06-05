import { GameState } from '../state.js';
import { GlobalTransform, LocalTransform } from '../transform.js';
import { GameNode } from './game-node.js';

export class SpriteNode<SA> implements GameNode<SA> {
  transform: LocalTransform;

  #globalTransform: GlobalTransform;
  #assetId: SA;

  constructor(assetId: SA) {
    this.transform = new LocalTransform();
    this.#globalTransform = new GlobalTransform();
    this.#assetId = assetId;
  }

  update(gameState: GameState<SA>, parentGlobalTransform: GlobalTransform): void {
    this.#globalTransform.update(parentGlobalTransform, this.transform);

    gameState.renderRequests.push({
      matrix: this.#globalTransform.matrix,
      alpha: this.#globalTransform.alpha,
      spriteAssetId: this.#assetId,
    });

    this.#globalTransform.finalize();
  }
}
