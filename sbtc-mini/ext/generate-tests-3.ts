import {
  Clarinet,
  Contract,
  Account,
} from "https://deno.land/x/clarinet@v1.7.0/index.ts";

const targetFolder = ".test-3";

const warningText = `// Code generated using \`clarinet run ./scripts/generate-tests.ts\`
// Manual edits will be lost.`;

function getContractName(contractId: string) {
  return contractId.split(".")[1];
}

function isTestContract(contractName: string) {
  return (
    contractName.substring(contractName.length - 10) === "_flow_test" &&
    contractName === "sbtc-stacking-pool_flow_test"
  );
}

const functionRegex =
  /^([ \t]{0,};;[ \t]{0,}@[\s\S]+?)\n[ \t]{0,}\(define-public[\s]+\((test-.+?)[ \t|)]/gm;
const annotationsRegex = /^;;[ \t]{1,}@([a-z-]+)(?:$|[ \t]+?(.+?))$/;
const callRegex =
  /\n*^([ \t]{0,};;[ \t]{0,}@[\s\S]+?)\n[ \t]{0,}(\((?:[^()]*|\([^()]*\))*\))/gm;

function extractTestAnnotationsAndCalls(contractSource: string) {
  const functionAnnotations = {};
  const functionBodies = {};
  contractSource = contractSource.replace(/\r/g, "");
  const matches1 = contractSource.matchAll(functionRegex);

  let indexStart: number = -1;
  let headerLength: number = 0;
  let indexEnd: number = -1;
  let lastFunctionName: string = "";
  let contractCalls: {
    callAnnotations: FunctionAnnotations;
    callInfo: CallInfo;
  }[];
  for (const [functionHeader, comments, functionName] of matches1) {
    if (functionName.substring(0, 5) !== "test-") continue;
    functionAnnotations[functionName] = {};
    const lines = comments.split("\n");
    for (const line of lines) {
      const [, prop, value] = line.match(annotationsRegex) || [];
      if (prop) functionAnnotations[functionName][prop] = value ?? true;
    }
    if (indexStart < 0) {
      indexStart = contractSource.indexOf(functionHeader);
      headerLength = functionHeader.length;
      lastFunctionName = functionName;
    } else {
      indexEnd = contractSource.indexOf(functionHeader);
      const lastFunctionBody = contractSource.substring(
        indexStart + headerLength,
        indexEnd
      );

      // add contracts calls in functions body for last function
      contractCalls = extractContractCalls(lastFunctionBody);

      functionBodies[lastFunctionName] = contractCalls;
      indexStart = indexEnd;
      headerLength = functionHeader.length;
      lastFunctionName = functionName;
    }
  }
  console.log({ indexStart, lastFunctionName, headerLength });
  const lastFunctionBody = contractSource.substring(indexStart + headerLength);
  contractCalls = extractContractCalls(lastFunctionBody);
  functionBodies[lastFunctionName] = contractCalls;

  console.log(functionBodies);
  return [functionAnnotations, functionBodies];
}

function extractContractCalls(lastFunctionBody: string) {
  const calls = lastFunctionBody.matchAll(callRegex);
  const contractCalls: {
    callAnnotations: FunctionAnnotations;
    callInfo: CallInfo;
  }[] = [];
  for (const [, comments, call] of calls) {
    const callAnnotations = {};
    const lines = comments.split("\n");
    for (const line of lines) {
      const [, prop, value] = line.match(annotationsRegex) || [];
      if (prop) callAnnotations[prop] = value ?? true;
    }
    let callInfo = extractUnwrapInfo(call);
    if (!callInfo) {
      callInfo = extractCallInfo(call);
    }
    if (callInfo) {
      contractCalls.push({ callAnnotations, callInfo });
    } else {
      throw new Error(`Could not extract call info from ${call}`);
    }
  }
  return contractCalls;
}

function extractUnwrapInfo(statement: string): CallInfo | null {
  const match = statement.match(/\(unwrap! \(contract-call\? \.(.+?) (.+?)(( .+?)*)\)/);
  if (!match) return null;

  const contractName = match[1];
  const functionName = match[2];
  const argStrings = match[3].split(" ").filter(Boolean);
  const args = argStrings.map((arg) => {
    const uintMatch = arg.match(/u(\d+)/);
    if (uintMatch) {
      return { type: "uint", value: uintMatch[1] };
    } else if (arg === "none") {
      return { type: "none", value: arg };
    }
    return { type: "raw", value: arg };
  });

  return {
    contractName,
    functionName,
    args,
  };
}

function extractCallInfo(statement: string) {
  const match = statement.match(/\((.+?)\)/);
  if (!match) return null;
  return { contractName: "", functionName: match[1], args: [] };
}

Clarinet.run({
  async fn(accounts: Map<string, Account>, contracts: Map<string, Contract>) {
    Deno.writeTextFile(`${targetFolder}/deps.ts`, generateDeps());

    for (const [contractId, contract] of contracts) {
      const contractName = getContractName(contractId);
      if (!isTestContract(contractName)) continue;

      const hasDefaultPrepareFunction =
        contract.contract_interface.functions.reduce(
          (a, v) =>
            a ||
            (v.name === "prepare" &&
              v.access === "public" &&
              v.args.length === 0),
          false
        );
      const [annotations, functionBodies] = extractTestAnnotationsAndCalls(
        contract.source
      );

      const code: string[][] = [];
      code.push([
        warningText,
        ``,
        `import { Clarinet, Tx, Chain, Account, types, assertEquals, printEvents, bootstrap } from './deps.ts';`,
        ``,
      ]);

      for (const {
        name,
        access,
        args,
      } of contract.contract_interface.functions.reverse()) {
        // is test function
        if (access !== "public" || name.substring(0, 5) !== "test-") continue;
        if (args.length > 0)
          throw new Error(
            `Test functions cannot take arguments. (Offending function: ${name})`
          );
        const functionAnnotations = annotations[name] || {};
        // update prepare annotation
        if (hasDefaultPrepareFunction && !functionAnnotations.prepare)
          functionAnnotations.prepare = "prepare";
        if (functionAnnotations["no-prepare"])
          delete functionAnnotations.prepare;

        const functionBody = functionBodies[name] || [];
        code.push([
          generateTest(contractId, name, functionAnnotations, functionBody),
        ]);
      }

      Deno.writeTextFile(
        `${targetFolder}/${contractName}.ts`,
        code.flat().join("\n")
      );
    }
  },
});

type FunctionAnnotations = { [key: string]: string | boolean };
type FunctionBody = {
  callAnnotations: FunctionAnnotations[];
  callInfo: CallInfo;
}[];

type CallInfo = {
  contractName: string;
  functionName: string;
  args: { type: string; value: string }[];
};

function generatePrepareTx(
  contractPrincipal: string,
  annotations: FunctionAnnotations
) {
  return `Tx.contractCall('${contractPrincipal}', '${annotations["prepare"]}', [], deployer.address)`;
}

function generateTxString(
  callInfo: CallInfo,
  contractPrincipal: string
): string {
  let argStrings = callInfo.args
    .map((arg) => {
      if (arg.type === "uint") {
        return `types.uint(${arg.value})`;
      } else {
        return `"${arg.value}"`; // Modify this to handle other types as needed
      }
    })
    .join(", ");

  return `Tx.contractCall("${callInfo.contractName || contractPrincipal}", "${
    callInfo.functionName
  }", [${argStrings}], callerAddress)
  `;
}

function generateNormalMineBlock(
  contractPrincipal: string,
  testFunction: string,
  annotations: FunctionAnnotations
) {
  return `let block = chain.mineBlock([
		${
      annotations["prepare"]
        ? `${generatePrepareTx(contractPrincipal, annotations)},`
        : ""
    }
		Tx.contractCall('${contractPrincipal}', '${testFunction}', [], callerAddress)
	]);`;
}

function generateSpecialMineBlock(
  mineBlocksBefore: number,
  contractPrincipal: string,
  testFunction: string,
  annotations: FunctionAnnotations
) {
  let code = ``;
  if (annotations["prepare"]) {
    code = `let prepareBlock = chain.mineBlock([${generatePrepareTx(
      contractPrincipal,
      annotations
    )}]);
		prepareBlock.receipts.map(({result}) => result.expectOk());
		`;
    if (annotations["print"] === "events")
      code += `\n\t\tprintEvents(prepareBlock);\n`;
  }
  if (mineBlocksBefore > 1)
    code += `
		chain.mineEmptyBlock(${mineBlocksBefore - 1});`;
  return `${code}
		let block = chain.mineBlock([Tx.contractCall('${contractPrincipal}', '${testFunction}', [], callerAddress)]);
		${annotations["print"] === "events" ? "printEvents(block);" : ""}`;
}

function generateBlocks(contractPrincipal: string, calls: FunctionBody) {
  let code = "";
  let blockStarted = false;
  for (const { callAnnotations, callInfo } of calls) {
    // mine empty blocks
    const mineBlocksBefore =
      parseInt(callAnnotations["mine-blocks-before"] as string) || 0;
    if (mineBlocksBefore > 1) {
      if (blockStarted) {
        code += `
			  ]);
			  block.receipts.map(({result}) => result.expectOk());
			  `;
        blockStarted = false;
      }
      code += `
			  chain.mineEmptyBlock(${mineBlocksBefore - 1});`;
    }
    // start a new block if necessary
    if (!blockStarted) {
      code += `
			  block = chain.mineBlock([`;
      blockStarted = true;
    }
    // add tx to current block
    code += generateTxString(callInfo, contractPrincipal);
    code += `,
	`;
  }
  // close final block
  if (blockStarted) {
    code += `
	  ]);
	  block.receipts.map(({result}) => result.expectOk());
	  `;
    blockStarted = false;
  }
  return code;
}

function generateFlowTest(
  contractPrincipal: string,
  testFunction: string,
  annotations: FunctionAnnotations,
  body: FunctionBody
) {
  return `Clarinet.test({
	name: "${
    annotations.name
      ? testFunction + ": " + (annotations.name as string).replace(/"/g, '\\"')
      : testFunction
  }",
	async fn(chain: Chain, accounts: Map<string, Account>) {
		const deployer = accounts.get("deployer")!;
		bootstrap(chain, deployer);
		let callerAddress = ${
      annotations.caller
        ? annotations.caller[0] === "'"
          ? `"${(annotations.caller as string).substring(1)}"`
          : `accounts.get('${annotations.caller}')!.address`
        : `accounts.get('deployer')!.address`
    };
		let block;
		${generateBlocks(contractPrincipal, body)}
	}
});
`;
}

function generateTest(
  contractPrincipal: string,
  testFunction: string,
  annotations: FunctionAnnotations,
  body: FunctionBody
) {
  console.log({ annotations });
  return generateFlowTest(contractPrincipal, testFunction, annotations, body);
}

function generateDeps() {
  return `${warningText}
	
import { Clarinet, Tx, Chain, Account, Block, types } from 'https://deno.land/x/clarinet@v1.5.4/index.ts';
import { assertEquals } from 'https://deno.land/std@0.170.0/testing/asserts.ts';

export { Clarinet, Tx, Chain, types, assertEquals };
export type { Account };

const dirOptions = {strAbbreviateSize: Infinity, depth: Infinity, colors: true};

export function printEvents(block: Block) {
	block.receipts.map(({events}) => events && events.map(event => console.log(Deno.inspect(event, dirOptions))));
}

export const bootstrapContracts = [
	'.sbtc-token',
	'.sbtc-peg-in-processor',
	'.sbtc-peg-out-processor',
	'.sbtc-registry',
	'.sbtc-stacking-pool',
	'.sbtc-testnet-debug-controller',
	'.sbtc-token'
];

export function bootstrap(chain: Chain, deployer: Account) {
	const { receipts } = chain.mineBlock([
		Tx.contractCall(
			\`\${deployer.address}.sbtc-controller\`,
			'upgrade',
			[types.list(bootstrapContracts.map(contract => types.tuple({ contract, enabled: true })))],
			deployer.address
		)
	]);
	receipts[0].result.expectOk().expectList().map(result => result.expectBool(true));
}`;
}
