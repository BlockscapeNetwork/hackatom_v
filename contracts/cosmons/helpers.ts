/*
 * This is a set of helpers meant for use with @cosmjs/cli
 * With these you can easily use the cw721 contract without worrying about forming messages and parsing queries.
 * 
 * Usage: npx @cosmjs/cli@^0.22 --init https://raw.githubusercontent.com/CosmWasm/cosmwasm-plus/master/contracts/cw20-base/helpers.ts
 * 
 * Create a client:
 *   const client = await useOptions(defaultOptions).setup(password);
 *   await client.getAccount()
 * 
 * Get the mnemonic:
 *   await useOptions(coralnetOptions).recoverMnemonic(password)
 * 
 * If you want to use this code inside an app, you will need several imports from https://github.com/CosmWasm/cosmjs
 */

const path = require("path");

interface Options {
  readonly httpUrl: string
  readonly networkId: string
  readonly feeToken: string
  readonly gasPrice: GasPrice
  readonly bech32prefix: string
  readonly hdPath: readonly Slip10RawIndex[]
  readonly faucetToken: string
  readonly faucetUrl?: string
  readonly defaultKeyFile: string
  readonly gasLimits: Partial<GasLimits<CosmWasmFeeTable>>
}

const defaultOptions: Options = {
  httpUrl: 'https://rest.cosmwasm.hub.hackatom.dev',
  networkId: 'hackatom-wasm',
  feeToken: 'ucosm',
  gasPrice: GasPrice.fromString("0.025ucosm"),
  bech32prefix: 'cosmos',
  faucetToken: 'COSM',
  faucetUrl: 'https://faucet.cosmwasm.hub.hackatom.dev/credit',
  hdPath: makeCosmoshubPath(0),
  defaultKeyFile: path.join(process.env.HOME, ".hackatom.key"),
  gasLimits: {
    upload: 1500000,
    init: 600000,
    register: 800000,
    transfer: 80000,
  },
}

const localnetOptions: Options = {
  httpUrl: "http://localhost:1317",
  networkId: 'localnet',
  feeToken: 'ucosm',
  gasPrice: GasPrice.fromString("0.025ucosm"),
  bech32prefix: 'cosmos',
  hdPath: makeCosmoshubPath(0),
  faucetToken: "SHELL",
  faucetUrl: "http://localhost",
  defaultKeyFile: path.join(process.env.HOME, "localnet.key"),
  gasLimits: {
    upload: 1500000,
    init: 600000,
    register: 800000,
    transfer: 80000,
  },
}

interface Network {
  setup: (password: string, filename?: string) => Promise<SigningCosmWasmClient>
  recoverMnemonic: (password: string, filename?: string) => Promise<string>
}

const useOptions = (options: Options): Network => {

  const loadOrCreateWallet = async (options: Options, filename: string, password: string): Promise<Secp256k1HdWallet> => {
    let encrypted: string;
    try {
      encrypted = fs.readFileSync(filename, 'utf8');
    } catch (err) {
      // generate if no file exists
      const wallet = await Secp256k1HdWallet.generate(12, options.hdPath, options.bech32prefix);
      const encrypted = await wallet.serialize(password);
      fs.writeFileSync(filename, encrypted, 'utf8');
      return wallet;
    }
    // otherwise, decrypt the file (we cannot put deserialize inside try or it will over-write on a bad password)
    const wallet = await Secp256k1HdWallet.deserialize(encrypted, password);
    return wallet;
  };

  const buildFeeTable = (feeToken: string, gasPrice: number): CosmWasmFeeTable => {
    const stdFee = (gas: number, denom: string, price: number) => {
      const amount = Math.floor(gas * price)
      return {
        amount: [{ amount: amount.toString(), denom: denom }],
        gas: gas.toString(),
      }
    }

    return {
      upload: stdFee(1000000, feeToken, gasPrice),
      init: stdFee(500000, feeToken, gasPrice),
      migrate: stdFee(500000, feeToken, gasPrice),
      exec: stdFee(200000, feeToken, gasPrice),
      send: stdFee(80000, feeToken, gasPrice),
      changeAdmin: stdFee(80000, feeToken, gasPrice),
    }
  }
  const connect = async (
    wallet: Secp256k1HdWallet,
    options: Options
  ): Promise<SigningCosmWasmClient> => {
    // const feeTable = buildFeeTable(options);
    // const feeTable = buildFeeTable(options.feeToken, options.gasPrice)
    const [{ address }] = await wallet.getAccounts();

    const client = new SigningCosmWasmClient(
      options.httpUrl,
      address,
      wallet,
      options.gasPrice,
      options.gasLimits,
    );
    return client;
  };

  const hitFaucet = async (
    faucetUrl: string,
    address: string,
    denom: string
  ): Promise<void> => {
    await axios.post(faucetUrl, { denom, address });
  }

  const setup = async (password: string, filename?: string): Promise<SigningCosmWasmClient> => {
    const keyfile = filename || options.defaultKeyFile;
    const wallet = await loadOrCreateWallet(options, keyfile, password);
    const client = await connect(wallet, options);

    // ensure we have some tokens
    if (options.faucetUrl) {
      const account = await client.getAccount();
      if (!account) {
        console.log(`Getting ${options.feeToken} from faucet`);
        await hitFaucet(options.faucetUrl, client.senderAddress, options.faucetToken);
      }
    }

    return client;
  }

  const recoverMnemonic = async (password: string, filename?: string): Promise<string> => {
    const keyfile = filename || options.defaultKeyFile;
    const wallet = await loadOrCreateWallet(options, keyfile, password);
    return wallet.mnemonic;
  }

  return { setup, recoverMnemonic };
}

type TokenId = string

interface Balances {
  readonly address: string
  readonly amount: string  // decimal as string
}

interface MintInfo {
  readonly minter: string
  readonly cap?: string // decimal as string
}

interface ContractInfo {
  readonly name: string
  readonly symbol: string
}

interface InitMsg {
  readonly name: string
  readonly symbol: string
  readonly minter: string
}
// Better to use this interface?
interface MintMsg {
  readonly token_id: TokenId
  readonly owner: string
  readonly name: string
  readonly description?: string
  readonly image?: string
}

type Expiration = { readonly at_height: number } | { readonly at_time: number } | { readonly never: {} };

interface AllowanceResponse {
  readonly allowance: string;  // integer as string
  readonly expires: Expiration;
}

interface AllowanceInfo {
  readonly allowance: string;  // integer as string
  readonly spender: string; // bech32 address
  readonly expires: Expiration;
}

interface AllAllowancesResponse {
  readonly allowances: readonly AllowanceInfo[];
}

interface AllAccountsResponse {
  // list of bech32 address that have a balance
  readonly accounts: readonly string[];
}

interface TokensResponse {
  readonly tokens: readonly string[];
}

interface CW721Instance {
  readonly contractAddress: string

  // queries
  // balance: (address?: string) => Promise<string>
  allowance: (owner: string, spender: string) => Promise<AllowanceResponse>
  allAllowances: (owner: string, startAfter?: string, limit?: number) => Promise<AllAllowancesResponse>
  allAccounts: (startAfter?: string, limit?: number) => Promise<readonly string[]>
  contractInfo: () => Promise<any>
  ownerOf: (tokenId: TokenId) => Promise<any>
  nftInfo: (tokenId: TokenId) => Promise<any>
  allNftInfo: (tokenId: TokenId) => Promise<any>
  // tokenInfo: () => Promise<any>
  minter: () => Promise<any>
  numTokens: () => Promise<any>
  tokens: (owner: string, startAfter?: string, limit?: number) => Promise<TokensResponse>
  allTokens: (startAfter?: string, limit?: number) => Promise<TokensResponse>
  approvedForAll: (owner: string, include_expired?: boolean, start_after?: string, limit?: number) => Promise<any>

  // actions
  mint: (tokenId: TokenId, owner: string, name: string, level: number, description?: string, image?: string) => Promise<string>
  battleMonster: (attacker_id: TokenId, defender_id: TokenId) => Promise<any>
  transferNft: (recipient: string, tokenId: TokenId) => Promise<string>
  sendNft: (contract: string, token_id: TokenId, msg?: BinaryType) => Promise<string>
  approve: (spender: string, tokenId: TokenId, expires?: Expiration) => Promise<string>
  approveAll: (operator: string, expires?: Expiration) => Promise<string>
  revoke: (spender: string, tokenId: TokenId) => Promise<string>
  revokeAll: (operator: string) => Promise<string>
  // burn: (amount: string) => Promise<string>
  // increaseAllowance: (recipient: string, amount: string) => Promise<string>
  // decreaseAllowance: (recipient: string, amount: string) => Promise<string>
  // transferFrom: (owner: string, recipient: string, amount: string) => Promise<string>
}

interface CW721Contract {
  // upload a code blob and returns a codeId
  upload: () => Promise<number>

  // instantiates a cw721 contract
  // codeId must come from a previous deploy
  // label is the public name of the contract in listing
  // if you set admin, you can run migrations on this contract (likely client.senderAddress)
  instantiate: (codeId: number, initMsg: Record<string, unknown>, label: string, admin?: string) => Promise<CW721Instance>

  use: (contractAddress: string) => CW721Instance
}


const CW721 = (client: SigningCosmWasmClient): CW721Contract => {
  const use = (contractAddress: string): CW721Instance => {
    /*    
        const balance = async (account?: string): Promise<string> => {
          const address = account || client.senderAddress;  
          const result = await client.queryContractSmart(contractAddress, {balance: { address }});
          return result.balance;
        };
    */
    const allowance = async (owner: string, spender: string): Promise<AllowanceResponse> => {
      return client.queryContractSmart(contractAddress, { allowance: { owner, spender } });
    };

    const allAllowances = async (owner: string, startAfter?: string, limit?: number): Promise<AllAllowancesResponse> => {
      return client.queryContractSmart(contractAddress, { all_allowances: { owner, start_after: startAfter, limit } });
    };

    const allAccounts = async (startAfter?: string, limit?: number): Promise<readonly string[]> => {
      const accounts: AllAccountsResponse = await client.queryContractSmart(contractAddress, { all_accounts: { start_after: startAfter, limit } });
      return accounts.accounts;
    };

    const minter = async (): Promise<any> => {
      return client.queryContractSmart(contractAddress, { minter: {} });
    };

    const contractInfo = async (): Promise<any> => {
      return client.queryContractSmart(contractAddress, { contract_info: {} });
    };

    const nftInfo = async (token_id: TokenId): Promise<any> => {
      return client.queryContractSmart(contractAddress, { nft_info: { token_id } });
    }

    const allNftInfo = async (token_id: TokenId): Promise<any> => {
      return client.queryContractSmart(contractAddress, { all_nft_info: { token_id } });
    }

    // TODO: Need help here
    const ownerOf = async (token_id: TokenId): Promise<any> => {
      return await client.queryContractSmart(contractAddress, { owner_of: { token_id } });
    }

    const approvedForAll = async (owner: string, include_expired?: boolean, start_after?: string, limit?: number): Promise<any> => {
      return await client.queryContractSmart(contractAddress, { approved_for_all: { owner, include_expired, start_after, limit } })
    }
    /*
        const tokenInfo = async (): Promise<any> => {
          return client.queryContractSmart(contractAddress, {token_info: { }});
        };
    */
    // mints tokens, returns ?
    const mint = async (token_id: TokenId, owner: string, name: string, level: number, description?: string, image?: string): Promise<string> => {
      const result = await client.execute(contractAddress, { mint: { token_id, owner, name, level, description, image } });
      return result.transactionHash;
    }

    // battle Monster
    const battleMonster = async (attacker_id: TokenId, defender_id: TokenId): Promise<string> => {
      const result = await client.execute(contractAddress, { battle_monster: { attacker_id, defender_id } });
      return result.transactionHash;
    }

    // transfers ownership, returns transactionHash
    const transferNft = async (recipient: string, token_id: TokenId): Promise<string> => {
      const result = await client.execute(contractAddress, { transfer_nft: { recipient, token_id } });
      return result.transactionHash;
    }

    // sends an nft token to another contract (TODO: msg type any needs to be revisited once receiveNft is implemented)
    const sendNft = async (contract: string, token_id: TokenId, msg?: any): Promise<string> => {
      const result = await client.execute(contractAddress, { send_nft: { contract, token_id, msg } })
      return result.transactionHash;
    }

    // total number of tokens issued
    const numTokens = async (): Promise<any> => {
      return client.queryContractSmart(contractAddress, { num_tokens: {} });
    }

    // list all token_ids that belong to a given owner
    const tokens = async (owner: string, start_after?: string, limit?: number): Promise<TokensResponse> => {
      return client.queryContractSmart(contractAddress, { tokens: { owner, start_after, limit } });
    }

    const allTokens = async (start_after?: string, limit?: number): Promise<TokensResponse> => {
      return client.queryContractSmart(contractAddress, { all_tokens: { start_after, limit } });
    }

    const approve = async (spender: string, token_id: TokenId, expires?: Expiration): Promise<string> => {
      const result = await client.execute(contractAddress, { approve: { spender, token_id, expires } });
      return result.transactionHash;
    }

    const approveAll = async (operator: string, expires?: Expiration): Promise<string> => {
      const result = await client.execute(contractAddress, { approve_all: { operator, expires } })
      return result.transactionHash
    }

    const revoke = async (spender: string, token_id: TokenId): Promise<string> => {
      const result = await client.execute(contractAddress, { revoke: { spender, token_id } });
      return result.transactionHash;
    }

    const revokeAll = async (operator: string): Promise<string> => {
      const result = await client.execute(contractAddress, { revoke_all: { operator } })
      return result.transactionHash;
    }
    /*
     // burns tokens, returns transactionHash
     const burn = async (amount: string): Promise<string> => {
       const result = await client.execute(contractAddress, {burn: {amount}});
       return result.transactionHash;
     }
 
     const increaseAllowance = async (spender: string, amount: string): Promise<string> => {
       const result = await client.execute(contractAddress, {increase_allowance: {spender, amount}});
       return result.transactionHash;
     }
 
     const decreaseAllowance = async (spender: string, amount: string): Promise<string> => {
       const result = await client.execute(contractAddress, {decrease_allowance: {spender, amount}});
       return result.transactionHash;
     }
 
     const transferFrom = async (owner: string, recipient: string, amount: string): Promise<string> => {
       const result = await client.execute(contractAddress, {transfer_from: {owner, recipient, amount}});
       return result.transactionHash;
     }
     */

    return {
      contractAddress,
      //balance,
      allowance,
      allAllowances,
      allAccounts,
      contractInfo,
      minter,
      mint,
      battleMonster,
      ownerOf,
      approvedForAll,
      nftInfo,
      allNftInfo,
      transferNft,
      sendNft,
      approve,
      approveAll,
      revoke,
      revokeAll,
      numTokens,
      tokens,
      allTokens
      // burn,
      // increaseAllowance,
      // decreaseAllowance,
      // transferFrom,
    };
  }

  const downloadWasm = async (url: string): Promise<Uint8Array> => {
    const r = await axios.get(url, { responseType: 'arraybuffer' })
    if (r.status !== 200) {
      throw new Error(`Download error: ${r.status}`)
    }
    return r.data
  }

  const upload = async (): Promise<number> => {
    const meta = {
      source: "https://github.com/CosmWasm/cosmwasm-plus/tree/v0.2.1/contracts/cw721-base",
      builder: "cosmwasm/workspace-optimizer:0.10.4"
    };
    const sourceUrl = "https://github.com/CosmWasm/cosmwasm-plus/releases/download/v0.2.1/cw721_base.wasm";
    const wasm = await downloadWasm(sourceUrl);
    const result = await client.upload(wasm, meta);
    return result.codeId;
  }

  const instantiate = async (codeId: number, initMsg: Record<string, unknown>, label: string, admin?: string): Promise<CW721Instance> => {
    const result = await client.instantiate(codeId, initMsg, label, { memo: `Init ${label}`, admin });
    return use(result.contractAddress);
  }

  return { upload, instantiate, use };
}

/*
const client = await useOptions(defaultOptions).setup("test");
const partner = await useOptions(defaultOptions).setup("test", ".localnet2.key");

const address = client.senderAddress;
const partnerAddr = partner.senderAddress;

const cw721 = CW721(client);

client.getAccount()
partner.getAccount()

const codeId = 94;

const initMsg = { name: "Cosmons", symbol: "mons",  minter: address };

const contract = await client.instantiate(codeId, initMsg, "Virtual Cosmons 1");

const mine = cw721.use(contract.contractAddress);

continue or stop here and jump to 2) 

mine.mint("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb", address, "Cosmos", "Minted Cosmon!");

mine.nftInfo("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb")


----
2)

mine.mint("monster112sarcsb", address, "Rustmorph", 20, "The Rustmorph is the coolest of the six original monster from Neptun");
mine.mint("monster112awrcsx", address, "Emberhand", 25, "The Rustmorph is the tallest of the five original monster from Saturn");

mine.nftInfo("monster112sarcsb")
mine.nftInfo("monster112awrcsx")

mine.battleMonster("monster112sarcsb","monster112awrcsx");

*/
