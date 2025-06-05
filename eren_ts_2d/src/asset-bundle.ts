import { GameState } from './state.js';

export class AssetBundle<SA> {
  #pendingSpriteAssets: Map<SA, string>;

  constructor(spriteAssets: Map<SA, string>) {
    this.#pendingSpriteAssets = spriteAssets;
  }

  isLoaded(gameState: GameState<SA>): boolean {
    if (this.#pendingSpriteAssets.size === 0) {
      return true;
    }

    const ready = gameState.spriteAssets.ready;
    const globalPending = gameState.spriteAssets.pending;

    this.#pendingSpriteAssets.forEach((path, asset) => {
      if (ready.has(asset)) {
        this.#pendingSpriteAssets.delete(asset);
        globalPending.delete(asset);
      } else if (!globalPending.has(asset)) {
        globalPending.set(asset, path);
      }
    });

    return this.#pendingSpriteAssets.size === 0;
  }
}
