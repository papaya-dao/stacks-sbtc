import { Clarinet, Contract, Account } from 'https://deno.land/x/clarinet@v1.5.4/index.ts';
const targetFolder = '.test';

const warningText = `// Code generated using \`clarinet run ./scripts/generate-tests.ts\`
// Manual edits will be lost.`;

function getContractName(contractId: string) {
    return contractId.split('.')[1];
}

function isTestContract(contractName: string) {
    return contractName.substring(contractName.length - 5) === "_test";
}

type UnwrapInfo = {
  contractName: string;
  functionName: string;
  args: {type: string, value: string}[];
};

function extractUnwrapInfo(statement: string): UnwrapInfo | null {
  const match = statement.match(/\(unwrap! \(contract-call\? \.(.+?) (.+?)(( .+?)*)\)/);
  if (!match) return null;

  const contractName = match[1];
  const functionName = match[2];
  const argStrings = match[3].split(' ').filter(Boolean);
  const args = argStrings.map(arg => {
      const uintMatch = arg.match(/u(\d+)/);
      if (uintMatch) {
          return { type: 'uint', value: uintMatch[1] };
      }
      return { type: 'raw', value: arg };
  });

  return {
      contractName,
      functionName,
      args
  };
}

function generateTxString(unwrapInfo: UnwrapInfo, sender: string): string {
  let argStrings = unwrapInfo.args.map(arg => {
      if(arg.type === 'uint') {
          return `types.uint(${arg.value})`;
      } else {
          return `"${arg.value}"`;  // Modify this to handle other types as needed
      }
  }).join(', ');

  return `Tx.contractCall("${unwrapInfo.contractName}", "${unwrapInfo.functionName}", [${argStrings}], ${sender})`;
}


const externalAnnotationsRegex = /^([ \t]{0,};;[ \t]{0,}@[\s\S]+?)\n[ \t]{0,}\(define-public[\s]+\((.+?)[ \t|)]/gm;
const annotationsRegex = /^;;[ \t]{1,}@([a-z-]+)(?:$|[ \t]+?(.+?))$/;

function extractExternalAnnotations(contractSource: string) {
    // Creating an object to store annotations with the function name as the key
    const externalFunctionAnnotations = {};
    // Replace all carriage returns (still unsure exactly what these are) with empty strings & then matching against the externalAnnotationsRegex
    const matches = contractSource.replace(/\r/g, "").matchAll(externalAnnotationsRegex);
    // For each match (function)
    for (const [, comments, functionName] of matches) {
        // Create an object specfic to the function
        externalFunctionAnnotations[functionName] = {};
        // Split the comments into an array of lines to handle multiple annotations
        const lines = comments.split("\n");
        // Loop through each line
        for (const line of lines) {
            // Match the line against the annotationsRegex to make sure it is an annotation
            const [, prop, value] = line.match(annotationsRegex) || [];
            // If the line is an annotation
            if (prop)
                // Add the annotation to the object
                externalFunctionAnnotations[functionName][prop] = value ?? true;
        }
    }
    //console.log("externalFunctionAnnotations with function name as key: " + externalFunctionAnnotations);
    return externalFunctionAnnotations;
}

// Each "Test" should be:
// External/Preparation Comments -> extractExternalAnnotations
// Function Signature (define-public (function-name args))
// Array of Comments or Functions
// Need to account for existing "prepare" function

Clarinet.run({
    async fn(accounts: Map<string, Account>, contracts: Map<string, Contract>) {
        Deno.writeTextFile(`${targetFolder}/deps.ts`, generateDeps());

        for (const [contractId, contract] of contracts) {
            const contractName = getContractName(contractId);
            if (contractName === "sbtc-stacking-pool_test") {
                const externalAnnotations = extractExternalAnnotations(contract.source);
                const code: string[][] = [];
                code.push([
                    warningText,
                    ``,
                    `import { Clarinet, Tx, Chain, Account, types, assertEquals, printEvents, bootstrap } from './deps.ts';`,
                    ``
                ]);
                //console.log(contract.contract_interface.functions.reverse())
                //console.log(contract.source)
                const testFunctions = extractTestFunctions(contract.source)
                let i = 0;
                //console.log("test functions: " + testFunctions);
                for (const functionName in testFunctions) {
                  const functionObject = testFunctions[functionName];
                  code.push([
                    `Clarinet.test({
                  name: "${functionName}",
                  async fn(chain: Chain, accounts: Map<string, Account>) {
                      const deployer = accounts.get("deployer")!;
                      bootstrap(chain, deployer);`
                  ]);
                  for(const event of functionObject.events) {
                      if(event.type === "unwrap") {
                          const unwrapInfo = extractUnwrapInfo(event.statement);
                          if (unwrapInfo) {
                              const txString = generateTxString(unwrapInfo, 'deployer.address');
                              code.push([
                                              `let block${i} = chain.mineBlock([
                                      ${txString}
                                  ]);`
                              ]);
                              i++;
                          }
                      } else if (event.type === "mine-empty") {
                          // Handle 'mine-empty'
                          code.push([
                            `chain.mineEmptyBlock(${event.blocks});`
                        ]);
                      }
                  }
                  code.push([
                    `block${i-1}.receipts.map(({result}) => result.expectOk());
    }
                            });`
                  ]);
              }



//                 for (const { name, access, args } of contract.contract_interface.functions.reverse()) {
//                     const functionAnnotations = externalAnnotations[name] || {};
//                     // console.log(name)
//                     // console.log(access)
//                     // console.log(args)
//                     code.push([
//                         `Clarinet.test({
//     name: "${name}",
//     async fn(chain: Chain, accounts: Map<string, Account>) {
//         const deployer = accounts.get("deployer")!;
//         bootstrap(chain, deployer);
//         let block = chain.mineBlock([
//             Tx.contractCall('${contractId}', '${name}', [], deployer.address)
//         ]);
//         block.receipts.map(({result}) => result.expectOk());
//     }
//                             });`
// ]);
//                 }

                console.log(code);

              Deno.writeTextFile(`${targetFolder}/${contractName}.ts`, code.flat().join("\n"));

            }
        }
    }
})

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

type FunctionEvent = { type: "unwrap", statement: string } | { type: "mine-empty", blocks: number };

type TestFunction = {
  events: FunctionEvent[],
};

type TestFunctions = {
  [key: string]: TestFunction
};

function extractTestFunctions(contractSource: string): TestFunctions {
  const lines = contractSource.split('\n');
  const testFunctions: TestFunctions = {};
  let currentFunctionName: string | null = null;
  let currentUnwrapStatement: string | null = null;
  let unwrapBlock = false;

  for (let line of lines) {
    line = line.trim();

    // Check for function definition
    const functionMatch = line.match(/^\(define-public\s+\((test-[^\s\)]+)/);
    if (functionMatch) {
      currentFunctionName = functionMatch[1];
      testFunctions[currentFunctionName] = {
        events: [],
      };
      continue;
    }

    // If we're inside a function...
    if (currentFunctionName) {
      // Check for unwrap statements
      if (unwrapBlock || line.startsWith('(unwrap!')) {
        unwrapBlock = true;
        currentUnwrapStatement = (currentUnwrapStatement || '') + line + '\n';
        if (line.endsWith(')')) {
          testFunctions[currentFunctionName].events.push({type: "unwrap", statement: currentUnwrapStatement.trim()});
          currentUnwrapStatement = null;
          unwrapBlock = false;
        }
        continue;
      }

      // Check for mine-empty blocks
      const mineEmptyBlocksMatch = line.match(/^;;\s*@mine-empty\s*([0-9]+)/);
      if (mineEmptyBlocksMatch) {
        testFunctions[currentFunctionName].events.push({type: "mine-empty", blocks: parseInt(mineEmptyBlocksMatch[1])});
        continue;
      }

      // Check for function end
      if (line === ')') {
        currentFunctionName = null;
      }
    }
  }
  //console.log("testFunction result: " + JSON.stringify(testFunctions, null, 2));
  return testFunctions;
}

//console.log(extractTestFunctions(contractSource));
//console.log("testFunction result: " + JSON.stringify(testFunctions, null, 2));