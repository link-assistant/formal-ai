export interface MemoryEvent { role?: string; content?: string; kind?: string; [key: string]: unknown }
export interface SolveContext { history?: MemoryEvent[]; preferences?: Record<string, unknown>; userContext?: Record<string, unknown> }
export interface Answer { content: string; intent: string; evidence: string[]; confidence: number; engine: string; [key: string]: unknown }
export interface Engine {
  solve(prompt: string, context?: SolveContext): Promise<Answer>;
  memory: {
    list(): Promise<MemoryEvent[]>;
    append(event: MemoryEvent): Promise<unknown>;
    clear(): Promise<number>;
    exportBundle(): Promise<string>;
    importBundle(text: string): Promise<Record<string, unknown>>;
  };
  dispose(): void;
}
export function createEngine(options?: { assetBase?: string | URL; timeoutMs?: number }): Promise<Engine>;
