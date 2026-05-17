import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import { verifyAction, VerifyActionRequest } from "../src/index";

test("verifyAction accepts the V0 flight-booking vector", () => {
  const response = verifyAction(acceptedRequest());

  assert.equal(response.accepted, true);
  assert.equal(response.rejection, null);
});

test("verifyAction preserves Rust verifier rejection codes", () => {
  const request = acceptedRequest();
  const action = request.action as { constraints: { amount_usd: { integer: number } } };
  action.constraints.amount_usd.integer = 900;

  const response = verifyAction(request);

  assert.equal(response.accepted, false);
  assert.equal(response.rejection?.code, "action_signature_invalid");
});

function acceptedRequest(): VerifyActionRequest {
  const vector = path.join(repositoryRoot(), "test-vectors/v0/flight-booking");
  const keys = readJson<{
    actor_public_key_hex: string;
    issuer_public_keys: Record<string, string>;
  }>(path.join(vector, "keys.json"));
  return {
    action: readJson(path.join(vector, "action.json")),
    capability_chain: readJson<VerifyActionRequest["capability_chain"]>(
      path.join(vector, "capability-chain.json")
    ),
    actor_public_key_hex: keys.actor_public_key_hex,
    issuer_public_keys: keys.issuer_public_keys,
    now_unix: 1650000000
  };
}

function readJson<T = VerifyActionRequest["action"]>(filePath: string): T {
  return JSON.parse(fs.readFileSync(filePath, "utf8")) as T;
}

function repositoryRoot(): string {
  return path.resolve(__dirname, "../../../..");
}
