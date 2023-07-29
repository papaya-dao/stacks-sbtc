export function getContractName(contractId: string) {
	return contractId.split('.')[1];
}

export function isTestContract(contractName: string) {
	return contractName.substring(contractName.length - 5) === "_test";
}

export function exitWithError(...args: any) {
	console.error(...args);
	Deno.exit(1);
}
