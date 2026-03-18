export interface RuntimeState {
  name: string;
  depth: number;
  index: number;
  nodeEnv: string;
}

export declare function loadRuntimeState(env?: Record<string, string | undefined>): RuntimeState;
export declare function listDependencyExports(): Array<{
  name: string;
  entry: string;
}>;
