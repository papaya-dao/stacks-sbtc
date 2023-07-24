import { Clarinet, Contract, Account } from 'https://deno.land/x/clarinet@v1.7.0/index.ts';

const targetFolder = '.mermaid';

function getContractName(contractId: string) {
    return contractId.split('.')[1];
}

Clarinet.run({
    async fn(accounts: Map<string, Account>, contracts: Map<string, Contract>) {

        const code: string[] = [];
        code.push("graph TD");

        // A map of maps to hold the function calls between contracts
        const contractCallsMap: Map<string, Map<string, Set<string>>> = new Map();

        for (const [contractId, contract] of contracts) {
            const contractName = getContractName(contractId);
            
            // Skip contracts with '_test' in the name
            if (contractName.includes('_test')) continue;

            const contractCalls = extractContractCalls(contract.source);
            for (const call of contractCalls) {
                // If the source contract is not in the map, add it
                if (!contractCallsMap.has(contractName)) {
                    contractCallsMap.set(contractName, new Map());
                }

                // If the target contract is not in the map, add it
                if (!contractCallsMap.get(contractName)!.has(call.contractName)) {
                    contractCallsMap.get(contractName)!.set(call.contractName, new Set());
                }

                // Add the function call to the map
                contractCallsMap.get(contractName)!.get(call.contractName)!.add(call.functionName);
            }
        }

        // Iterate over the contract calls map to generate the Mermaid code
        for (const [sourceContract, targetMap] of contractCallsMap) {
            for (const [targetContract, functionNames] of targetMap) {
                let functionNamesArray = Array.from(functionNames);
                let functionNamesToDisplay = functionNamesArray.slice(0, 5);
                let remainingFunctions = functionNamesArray.length - 5;

                if (remainingFunctions > 0) {
                    functionNamesToDisplay.push(`+ ${remainingFunctions} more functions`);
                }

                // Display the function names
                code.push(`  ${sourceContract} -->|${functionNamesToDisplay.join("\\n")}| ${targetContract}`);
            }
        }

        Deno.writeTextFile(`${targetFolder}/generated-mermaid.mmd`, code.join("\n"));
    }
});

type ContractCall = {
    contractName: string;
    functionName: string;
};

function extractContractCalls(contractSource: string): ContractCall[] {
    const contractCalls: ContractCall[] = [];
    const regex = /\(contract-call\? \.(.+?) (.+?)(( .+?)*)\)/g;
    let match;

    while ((match = regex.exec(contractSource)) !== null) {
        const contractName = match[1];
        const functionName = match[2];
        contractCalls.push({ contractName, functionName });
    }

    return contractCalls;
}
