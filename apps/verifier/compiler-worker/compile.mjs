import process from "node:process";

import { compileFunc } from "./languages/func.mjs";
import { compileTact } from "./languages/tact.mjs";
import { compileTolk } from "./languages/tolk.mjs";
import {
  readStdin,
  validateCompileInput,
  validateInput,
  writeOutput,
} from "./languages/common.mjs";

try {
  const input = validateInput(JSON.parse(await readStdin(process.stdin)));
  switch (input.operation) {
    case "compile": {
      const compileInput = validateCompileInput(input);
      writeOutput(process.stdout, await compile(compileInput));
      break;
    }
    default:
      throw new Error(`unsupported operation: ${String(input?.operation)}`);
  }
} catch (error) {
  writeOutput(process.stdout, {
    status: "compile_error",
    error: error instanceof Error ? error.message : String(error),
  });
}

/**
 * @param {import("./languages/common.mjs").CompileInput} input
 */
async function compile(input) {
  switch (input.language) {
    case "func":
      return compileFunc(input);
    case "tact":
      return compileTact(input);
    case "tolk":
      return compileTolk(input);
    default:
      throw new Error(`unsupported language: ${input.language}`);
  }
}
