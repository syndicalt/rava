export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

export interface VerifyActionRequest {
  action: JsonValue;
  capability_chain: JsonValue[];
  actor_public_key_hex: string;
  issuer_public_keys: Record<string, string>;
  now_unix: number;
  revoked_ids?: string[];
}

export interface VerifyActionRejection {
  code: string;
  subject: string | null;
}

export interface VerifyActionResponse {
  accepted: boolean;
  rejection: VerifyActionRejection | null;
}

interface RavaWasmModule {
  verify_action_json(requestJson: string): string;
}

function loadWasm(): RavaWasmModule {
  return require("../wasm/rava_wasm.js") as RavaWasmModule;
}

export function verifyAction(request: VerifyActionRequest): VerifyActionResponse {
  const responseJson = loadWasm().verify_action_json(JSON.stringify(request));
  return JSON.parse(responseJson) as VerifyActionResponse;
}
