import { GameState } from '../state.js';
import { GlobalTransform } from '../transform.js';

export interface GameNode<SA> {
  update(gameState: GameState<SA>, parentGlobalTransform: GlobalTransform): void;
}
