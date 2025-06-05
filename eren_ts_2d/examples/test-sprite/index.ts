import { GameNode } from "../../dist/nodes/game_node.js";
import { GameState } from '../../dist/state.js';
import { GlobalTransform } from '../../dist/transform.js';
import { AssetBundle } from '../../dist/asset_bundle.js';
import { SpriteNode } from '../../dist/nodes/sprite_node.js';

enum SpriteAssets {
  Logo,
  TestSprite,
}

class RootNode implements GameNode<SpriteAssets> {
  #loadingScreen?: LoadingScreen;
  #inGameScreen: InGameScreen;

  constructor() {
    this.#loadingScreen = new LoadingScreen();
    this.#inGameScreen = new InGameScreen();
  }

  update(gameState: GameState<SpriteAssets>, parentGlobalTransform: GlobalTransform): void {
    if (this.#inGameScreen.assetBundle.isLoaded(gameState)) {
      this.#loadingScreen = undefined;
    }

    if (this.#loadingScreen) {
      this.#loadingScreen.update(gameState, parentGlobalTransform);
    } else {
      this.#inGameScreen.update(gameState, parentGlobalTransform);
    }
  }
}

class LoadingScreen implements GameNode<SpriteAssets> {
  #assetBundle: AssetBundle<SpriteAssets>;
  #logo: SpriteNode<SpriteAssets>;

  constructor() {
    this.#assetBundle = new AssetBundle<SpriteAssets>(new Map([
      [SpriteAssets.Logo, "examples/assets/logo.png"],
    ]));
    this.#logo = new SpriteNode(SpriteAssets.Logo);
  }

  update(gameState: GameState<SpriteAssets>, parentGlobalTransform: GlobalTransform): void {
    if (this.#assetBundle.isLoaded(gameState)) {
      this.#logo.update(gameState, parentGlobalTransform);
    }
  }
}

class InGameScreen implements GameNode<SpriteAssets> {
  assetBundle: AssetBundle<SpriteAssets>;
  #sprite: SpriteNode<SpriteAssets>;

  constructor() {
    this.assetBundle = new AssetBundle<SpriteAssets>(new Map([
      [SpriteAssets.TestSprite, "examples/assets/test_sprite.png"],
    ]));
    this.#sprite = new SpriteNode(SpriteAssets.TestSprite);
  }

  update(gameState: GameState<SpriteAssets>, parentGlobalTransform: GlobalTransform): void {
    if (this.assetBundle.isLoaded(gameState)) {
      this.#sprite.update(gameState, parentGlobalTransform);
    }
  }
}

// TODO: Add app
new RootNode();
