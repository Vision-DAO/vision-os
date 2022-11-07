/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import {
  Signer,
  utils,
  Contract,
  ContractFactory,
  BigNumberish,
  Overrides,
} from "ethers";
import type { Provider, TransactionRequest } from "@ethersproject/providers";
import type { PromiseOrValue } from "../../common";
import type { Idea, IdeaInterface } from "../../contracts/Idea";

const _abi = [
  {
    inputs: [
      {
        internalType: "string",
        name: "_name",
        type: "string",
      },
      {
        internalType: "string",
        name: "_symbol",
        type: "string",
      },
      {
        internalType: "uint256",
        name: "_supply",
        type: "uint256",
      },
      {
        internalType: "string",
        name: "_ipfsAddr",
        type: "string",
      },
    ],
    stateMutability: "nonpayable",
    type: "constructor",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "owner",
        type: "address",
      },
      {
        indexed: true,
        internalType: "address",
        name: "spender",
        type: "address",
      },
      {
        indexed: false,
        internalType: "uint256",
        name: "value",
        type: "uint256",
      },
    ],
    name: "Approval",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: false,
        internalType: "contract Proposal",
        name: "proposal",
        type: "address",
      },
      {
        indexed: false,
        internalType: "string",
        name: "oldPayload",
        type: "string",
      },
      {
        indexed: false,
        internalType: "string",
        name: "newPayload",
        type: "string",
      },
    ],
    name: "ProposalAccepted",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: false,
        internalType: "contract Proposal",
        name: "proposal",
        type: "address",
      },
    ],
    name: "ProposalRejected",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "from",
        type: "address",
      },
      {
        indexed: true,
        internalType: "address",
        name: "to",
        type: "address",
      },
      {
        indexed: false,
        internalType: "uint256",
        name: "value",
        type: "uint256",
      },
    ],
    name: "Transfer",
    type: "event",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "owner",
        type: "address",
      },
      {
        internalType: "address",
        name: "spender",
        type: "address",
      },
    ],
    name: "allowance",
    outputs: [
      {
        internalType: "uint256",
        name: "",
        type: "uint256",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "spender",
        type: "address",
      },
      {
        internalType: "uint256",
        name: "amount",
        type: "uint256",
      },
    ],
    name: "approve",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
    ],
    name: "balanceOf",
    outputs: [
      {
        internalType: "uint256",
        name: "",
        type: "uint256",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "voter",
        type: "address",
      },
      {
        components: [
          {
            internalType: "contract Proposal",
            name: "dependent",
            type: "address",
          },
          {
            internalType: "uint256",
            name: "weight",
            type: "uint256",
          },
          {
            internalType: "enum VoteKind",
            name: "nature",
            type: "uint8",
          },
        ],
        internalType: "struct Commitment",
        name: "vote",
        type: "tuple",
      },
    ],
    name: "commitVotes",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "contract Proposal",
        name: "prop",
        type: "address",
      },
      {
        internalType: "address",
        name: "voter",
        type: "address",
      },
    ],
    name: "commitment",
    outputs: [
      {
        components: [
          {
            internalType: "contract Proposal",
            name: "dependent",
            type: "address",
          },
          {
            internalType: "uint256",
            name: "weight",
            type: "uint256",
          },
          {
            internalType: "enum VoteKind",
            name: "nature",
            type: "uint8",
          },
        ],
        internalType: "struct Commitment",
        name: "",
        type: "tuple",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "decimals",
    outputs: [
      {
        internalType: "uint8",
        name: "",
        type: "uint8",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "spender",
        type: "address",
      },
      {
        internalType: "uint256",
        name: "subtractedValue",
        type: "uint256",
      },
    ],
    name: "decreaseAllowance",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "contract Proposal",
        name: "proposal",
        type: "address",
      },
    ],
    name: "finalizeProposal",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "spender",
        type: "address",
      },
      {
        internalType: "uint256",
        name: "addedValue",
        type: "uint256",
      },
    ],
    name: "increaseAllowance",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "ipfsAddr",
    outputs: [
      {
        internalType: "string",
        name: "",
        type: "string",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "name",
    outputs: [
      {
        internalType: "string",
        name: "",
        type: "string",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "symbol",
    outputs: [
      {
        internalType: "string",
        name: "",
        type: "string",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "totalSupply",
    outputs: [
      {
        internalType: "uint256",
        name: "",
        type: "uint256",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "to",
        type: "address",
      },
      {
        internalType: "uint256",
        name: "amount",
        type: "uint256",
      },
    ],
    name: "transfer",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "from",
        type: "address",
      },
      {
        internalType: "address",
        name: "to",
        type: "address",
      },
      {
        internalType: "uint256",
        name: "amount",
        type: "uint256",
      },
    ],
    name: "transferFrom",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
];

const _bytecode =
  "0x60806040523480156200001157600080fd5b506040516200374738038062003747833981810160405281019062000037919062000988565b838381600390805190602001906200005192919062000700565b5080600490805190602001906200006a92919062000700565b50505080600590805190602001906200008592919062000700565b50620000983383620000a260201b60201c565b5050505062000e08565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141562000115576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016200010c9062000ab8565b60405180910390fd5b62000129600083836200021b60201b60201c565b80600260008282546200013d919062000b09565b92505081905550806000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825462000194919062000b09565b925050819055508173ffffffffffffffffffffffffffffffffffffffff16600073ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef83604051620001fb919062000b77565b60405180910390a362000217600083836200022060201b60201c565b5050565b505050565b6000600660008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206002015414156200027257620006b3565b6000600660008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020905060008160020154905060005b81811015620006af576000836001018281548110620002e557620002e462000b94565b5b9060005260206000200160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905060008460000160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020905060006200036a89620006b860201b60201c565b90506000826001015414806200049057508273ffffffffffffffffffffffffffffffffffffffff1663e184c9be6040518163ffffffff1660e01b815260040160206040518083038186803b158015620003c257600080fd5b505afa158015620003d7573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190620003fd919062000bc3565b42101580156200048f575060008373ffffffffffffffffffffffffffffffffffffffff1663e184c9be6040518163ffffffff1660e01b815260040160206040518083038186803b1580156200045157600080fd5b505afa15801562000466573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906200048c919062000bc3565b14155b5b15620005c3578480620004a39062000bf5565b95505060008260010181905550856001018581548110620004c957620004c862000b94565b5b9060005260206000200160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff168660010185815481106200050d576200050c62000b94565b5b9060005260206000200160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550856001018054806200056c576200056b62000c24565b5b6001900381819060005260206000200160006101000a81549073ffffffffffffffffffffffffffffffffffffffff02191690559055856002016000815480929190620005b89062000bf5565b9190505550620006a6565b808660000160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600101541115620006a5578273ffffffffffffffffffffffffffffffffffffffff166343bd682d8a8460020160009054906101000a900460ff16846040518463ffffffff1660e01b8152600401620006609392919062000d18565b600060405180830381600087803b1580156200067b57600080fd5b505af115801562000690573d6000803e3d6000fd5b505050508380620006a19062000d55565b9450505b5b505050620002c1565b5050505b505050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b8280546200070e9062000dd2565b90600052602060002090601f0160209004810192826200073257600085556200077e565b82601f106200074d57805160ff19168380011785556200077e565b828001600101855582156200077e579182015b828111156200077d57825182559160200191906001019062000760565b5b5090506200078d919062000791565b5090565b5b80821115620007ac57600081600090555060010162000792565b5090565b6000604051905090565b600080fd5b600080fd5b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b6200081982620007ce565b810181811067ffffffffffffffff821117156200083b576200083a620007df565b5b80604052505050565b600062000850620007b0565b90506200085e82826200080e565b919050565b600067ffffffffffffffff821115620008815762000880620007df565b5b6200088c82620007ce565b9050602081019050919050565b60005b83811015620008b95780820151818401526020810190506200089c565b83811115620008c9576000848401525b50505050565b6000620008e6620008e08462000863565b62000844565b905082815260208101848484011115620009055762000904620007c9565b5b6200091284828562000899565b509392505050565b600082601f830112620009325762000931620007c4565b5b815162000944848260208601620008cf565b91505092915050565b6000819050919050565b62000962816200094d565b81146200096e57600080fd5b50565b600081519050620009828162000957565b92915050565b60008060008060808587031215620009a557620009a4620007ba565b5b600085015167ffffffffffffffff811115620009c657620009c5620007bf565b5b620009d4878288016200091a565b945050602085015167ffffffffffffffff811115620009f857620009f7620007bf565b5b62000a06878288016200091a565b935050604062000a198782880162000971565b925050606085015167ffffffffffffffff81111562000a3d5762000a3c620007bf565b5b62000a4b878288016200091a565b91505092959194509250565b600082825260208201905092915050565b7f45524332303a206d696e7420746f20746865207a65726f206164647265737300600082015250565b600062000aa0601f8362000a57565b915062000aad8262000a68565b602082019050919050565b6000602082019050818103600083015262000ad38162000a91565b9050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600062000b16826200094d565b915062000b23836200094d565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0382111562000b5b5762000b5a62000ada565b5b828201905092915050565b62000b71816200094d565b82525050565b600060208201905062000b8e600083018462000b66565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b60006020828403121562000bdc5762000bdb620007ba565b5b600062000bec8482850162000971565b91505092915050565b600062000c02826200094d565b9150600082141562000c195762000c1862000ada565b5b600182039050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603160045260246000fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600062000c808262000c53565b9050919050565b62000c928162000c73565b82525050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602160045260246000fd5b6002811062000cdb5762000cda62000c98565b5b50565b600081905062000cee8262000cc7565b919050565b600062000d008262000cde565b9050919050565b62000d128162000cf3565b82525050565b600060608201905062000d2f600083018662000c87565b62000d3e602083018562000d07565b62000d4d604083018462000b66565b949350505050565b600062000d62826200094d565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82141562000d985762000d9762000ada565b5b600182019050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b6000600282049050600182168062000deb57607f821691505b6020821081141562000e025762000e0162000da3565b5b50919050565b61292f8062000e186000396000f3fe608060405234801561001057600080fd5b50600436106100f55760003560e01c80633950935111610097578063a457c2d711610066578063a457c2d71461029c578063a9059cbb146102cc578063bc7bd0f6146102fc578063dd62ed3e1461032c576100f5565b806339509351146101ee57806370a082311461021e578063907602501461024e57806395d89b411461027e576100f5565b8063095ea7b3116100d3578063095ea7b31461015257806318160ddd1461018257806323b872dd146101a0578063313ce567146101d0576100f5565b806305df721e146100fa57806306fdde031461011857806308f1cacd14610136575b600080fd5b61010261035c565b60405161010f9190611966565b60405180910390f35b6101206103ea565b60405161012d9190611966565b60405180910390f35b610150600480360381019061014b9190611b77565b61047c565b005b61016c60048036038101906101679190611bb7565b61048b565b6040516101799190611c12565b60405180910390f35b61018a6104ae565b6040516101979190611c3c565b60405180910390f35b6101ba60048036038101906101b59190611c57565b6104b8565b6040516101c79190611c12565b60405180910390f35b6101d86104e7565b6040516101e59190611cc6565b60405180910390f35b61020860048036038101906102039190611bb7565b6104f0565b6040516102159190611c12565b60405180910390f35b61023860048036038101906102339190611ce1565b610527565b6040516102459190611c3c565b60405180910390f35b61026860048036038101906102639190611d0e565b61056f565b6040516102759190611c12565b60405180910390f35b61028661095d565b6040516102939190611966565b60405180910390f35b6102b660048036038101906102b19190611bb7565b6109ef565b6040516102c39190611c12565b60405180910390f35b6102e660048036038101906102e19190611bb7565b610a66565b6040516102f39190611c12565b60405180910390f35b61031660048036038101906103119190611d3b565b610a89565b6040516103239190611ea2565b60405180910390f35b61034660048036038101906103419190611ebd565b610bbe565b6040516103539190611c3c565b60405180910390f35b6005805461036990611f2c565b80601f016020809104026020016040519081016040528092919081815260200182805461039590611f2c565b80156103e25780601f106103b7576101008083540402835291602001916103e2565b820191906000526020600020905b8154815290600101906020018083116103c557829003601f168201915b505050505081565b6060600380546103f990611f2c565b80601f016020809104026020016040519081016040528092919081815260200182805461042590611f2c565b80156104725780601f1061044757610100808354040283529160200191610472565b820191906000526020600020905b81548152906001019060200180831161045557829003601f168201915b5050505050905090565b610487338383610c45565b5050565b600080610496610e93565b90506104a3818585610e9b565b600191505092915050565b6000600254905090565b6000806104c3610e93565b90506104d0858285611066565b6104db8585856110f2565b60019150509392505050565b60006012905090565b6000806104fb610e93565b905061051c81858561050d8589610bbe565b6105179190611f8d565b610e9b565b600191505092915050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b6000808273ffffffffffffffffffffffffffffffffffffffff16639ef27b006040518163ffffffff1660e01b815260040160206040518083038186803b1580156105b857600080fd5b505afa1580156105cc573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906105f09190611ff8565b14801561067a57508173ffffffffffffffffffffffffffffffffffffffff1663e184c9be6040518163ffffffff1660e01b815260040160206040518083038186803b15801561063e57600080fd5b505afa158015610652573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906106769190611ff8565b4210155b6106b9576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016106b090612097565b60405180910390fd5b8173ffffffffffffffffffffffffffffffffffffffff166348f854516040518163ffffffff1660e01b8152600401600060405180830381600087803b15801561070157600080fd5b505af1158015610715573d6000803e3d6000fd5b5050505060326107236104ae565b60648473ffffffffffffffffffffffffffffffffffffffff166364fd9a946040518163ffffffff1660e01b815260040160206040518083038186803b15801561076b57600080fd5b505afa15801561077f573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906107a39190611ff8565b6107ad91906120b7565b6107b79190612140565b116107fc577fb5daea123b4f9ceb812de7559307c1823da6779003495a79531735c4bd12d62e826040516107eb9190612180565b60405180910390a160009050610958565b7f832acad2f3effea3d65d4581773093880b8abc218f991fd2e413222ac50e66f48260058473ffffffffffffffffffffffffffffffffffffffff1663a878f8586040518163ffffffff1660e01b815260040160006040518083038186803b15801561086657600080fd5b505afa15801561087a573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f820116820180604052508101906108a39190612246565b6040516108b293929190612324565b60405180910390a18173ffffffffffffffffffffffffffffffffffffffff1663a878f8586040518163ffffffff1660e01b815260040160006040518083038186803b15801561090057600080fd5b505afa158015610914573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f8201168201806040525081019061093d9190612246565b600590805190602001906109529291906117e1565b50600190505b919050565b60606004805461096c90611f2c565b80601f016020809104026020016040519081016040528092919081815260200182805461099890611f2c565b80156109e55780601f106109ba576101008083540402835291602001916109e5565b820191906000526020600020905b8154815290600101906020018083116109c857829003601f168201915b5050505050905090565b6000806109fa610e93565b90506000610a088286610bbe565b905083811015610a4d576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610a44906123db565b60405180910390fd5b610a5a8286868403610e9b565b60019250505092915050565b600080610a71610e93565b9050610a7e8185856110f2565b600191505092915050565b610a91611867565b600660008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060000160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206040518060600160405290816000820160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001600182015481526020016002820160009054906101000a900460ff166001811115610ba057610b9f611de9565b5b6001811115610bb257610bb1611de9565b5b81525050905092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b806000015173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1614610cb7576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610cae9061246d565b60405180910390fd5b6000600660008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020905060008160000160008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600101541415610dc757806002016000815480929190610d5c9061248d565b919050555080600101849080600181540180825580915050600190039060005260206000200160009091909190916101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055505b818160000160008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008201518160000160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506020820151816001015560408201518160020160006101000a81548160ff02191690836001811115610e8557610e84611de9565b5b021790555090505050505050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415610f0b576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610f0290612548565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415610f7b576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610f72906125da565b60405180910390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925836040516110599190611c3c565b60405180910390a3505050565b60006110728484610bbe565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146110ec57818110156110de576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016110d590612646565b60405180910390fd5b6110eb8484848403610e9b565b5b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415611162576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401611159906126d8565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614156111d2576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016111c99061276a565b60405180910390fd5b6111dd838383611373565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905081811015611263576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161125a906127fc565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008282546112f69190611f8d565b925050819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8460405161135a9190611c3c565b60405180910390a361136d848484611378565b50505050565b505050565b6000600660008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206002015414156113c8576117dc565b6000600660008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020905060008160020154905060005b818110156117d85760008360010182815481106114375761143661281c565b5b9060005260206000200160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905060008460000160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020905060006114b489610527565b90506000826001015414806115d057508273ffffffffffffffffffffffffffffffffffffffff1663e184c9be6040518163ffffffff1660e01b815260040160206040518083038186803b15801561150a57600080fd5b505afa15801561151e573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906115429190611ff8565b42101580156115cf575060008373ffffffffffffffffffffffffffffffffffffffff1663e184c9be6040518163ffffffff1660e01b815260040160206040518083038186803b15801561159457600080fd5b505afa1580156115a8573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906115cc9190611ff8565b14155b5b156116f45784806115e09061284b565b955050600082600101819055508560010185815481106116035761160261281c565b5b9060005260206000200160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff168660010185815481106116445761164361281c565b5b9060005260206000200160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550856001018054806116a05761169f612875565b5b6001900381819060005260206000200160006101000a81549073ffffffffffffffffffffffffffffffffffffffff021916905590558560020160008154809291906116ea9061284b565b91905055506117d0565b808660000160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206001015411156117cf578273ffffffffffffffffffffffffffffffffffffffff166343bd682d8a8460020160009054906101000a900460ff16846040518463ffffffff1660e01b815260040161178e939291906128c2565b600060405180830381600087803b1580156117a857600080fd5b505af11580156117bc573d6000803e3d6000fd5b5050505083806117cb9061248d565b9450505b5b505050611417565b5050505b505050565b8280546117ed90611f2c565b90600052602060002090601f01602090048101928261180f5760008555611856565b82601f1061182857805160ff1916838001178555611856565b82800160010185558215611856579182015b8281111561185557825182559160200191906001019061183a565b5b50905061186391906118b0565b5090565b6040518060600160405280600073ffffffffffffffffffffffffffffffffffffffff16815260200160008152602001600060018111156118aa576118a9611de9565b5b81525090565b5b808211156118c95760008160009055506001016118b1565b5090565b600081519050919050565b600082825260208201905092915050565b60005b838110156119075780820151818401526020810190506118ec565b83811115611916576000848401525b50505050565b6000601f19601f8301169050919050565b6000611938826118cd565b61194281856118d8565b93506119528185602086016118e9565b61195b8161191c565b840191505092915050565b60006020820190508181036000830152611980818461192d565b905092915050565b6000604051905090565b600080fd5b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b60006119c78261199c565b9050919050565b6119d7816119bc565b81146119e257600080fd5b50565b6000813590506119f4816119ce565b92915050565b600080fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b611a378261191c565b810181811067ffffffffffffffff82111715611a5657611a556119ff565b5b80604052505050565b6000611a69611988565b9050611a758282611a2e565b919050565b6000611a85826119bc565b9050919050565b611a9581611a7a565b8114611aa057600080fd5b50565b600081359050611ab281611a8c565b92915050565b6000819050919050565b611acb81611ab8565b8114611ad657600080fd5b50565b600081359050611ae881611ac2565b92915050565b60028110611afb57600080fd5b50565b600081359050611b0d81611aee565b92915050565b600060608284031215611b2957611b286119fa565b5b611b336060611a5f565b90506000611b4384828501611aa3565b6000830152506020611b5784828501611ad9565b6020830152506040611b6b84828501611afe565b60408301525092915050565b60008060808385031215611b8e57611b8d611992565b5b6000611b9c858286016119e5565b9250506020611bad85828601611b13565b9150509250929050565b60008060408385031215611bce57611bcd611992565b5b6000611bdc858286016119e5565b9250506020611bed85828601611ad9565b9150509250929050565b60008115159050919050565b611c0c81611bf7565b82525050565b6000602082019050611c276000830184611c03565b92915050565b611c3681611ab8565b82525050565b6000602082019050611c516000830184611c2d565b92915050565b600080600060608486031215611c7057611c6f611992565b5b6000611c7e868287016119e5565b9350506020611c8f868287016119e5565b9250506040611ca086828701611ad9565b9150509250925092565b600060ff82169050919050565b611cc081611caa565b82525050565b6000602082019050611cdb6000830184611cb7565b92915050565b600060208284031215611cf757611cf6611992565b5b6000611d05848285016119e5565b91505092915050565b600060208284031215611d2457611d23611992565b5b6000611d3284828501611aa3565b91505092915050565b60008060408385031215611d5257611d51611992565b5b6000611d6085828601611aa3565b9250506020611d71858286016119e5565b9150509250929050565b6000819050919050565b6000611da0611d9b611d968461199c565b611d7b565b61199c565b9050919050565b6000611db282611d85565b9050919050565b6000611dc482611da7565b9050919050565b611dd481611db9565b82525050565b611de381611ab8565b82525050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602160045260246000fd5b60028110611e2957611e28611de9565b5b50565b6000819050611e3a82611e18565b919050565b6000611e4a82611e2c565b9050919050565b611e5a81611e3f565b82525050565b606082016000820151611e766000850182611dcb565b506020820151611e896020850182611dda565b506040820151611e9c6040850182611e51565b50505050565b6000606082019050611eb76000830184611e60565b92915050565b60008060408385031215611ed457611ed3611992565b5b6000611ee2858286016119e5565b9250506020611ef3858286016119e5565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680611f4457607f821691505b60208210811415611f5857611f57611efd565b5b50919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000611f9882611ab8565b9150611fa383611ab8565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff03821115611fd857611fd7611f5e565b5b828201905092915050565b600081519050611ff281611ac2565b92915050565b60006020828403121561200e5761200d611992565b5b600061201c84828501611fe3565b91505092915050565b7f50726f706f73616c20766f74696e6720706572696f6420686173206e6f74207960008201527f65742066696e69736865642e0000000000000000000000000000000000000000602082015250565b6000612081602c836118d8565b915061208c82612025565b604082019050919050565b600060208201905081810360008301526120b081612074565b9050919050565b60006120c282611ab8565b91506120cd83611ab8565b9250817fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff048311821515161561210657612105611f5e565b5b828202905092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601260045260246000fd5b600061214b82611ab8565b915061215683611ab8565b92508261216657612165612111565b5b828204905092915050565b61217a81611db9565b82525050565b60006020820190506121956000830184612171565b92915050565b600080fd5b600080fd5b600067ffffffffffffffff8211156121c0576121bf6119ff565b5b6121c98261191c565b9050602081019050919050565b60006121e96121e4846121a5565b611a5f565b905082815260208101848484011115612205576122046121a0565b5b6122108482856118e9565b509392505050565b600082601f83011261222d5761222c61219b565b5b815161223d8482602086016121d6565b91505092915050565b60006020828403121561225c5761225b611992565b5b600082015167ffffffffffffffff81111561227a57612279611997565b5b61228684828501612218565b91505092915050565b60008190508160005260206000209050919050565b600081546122b181611f2c565b6122bb81866118d8565b945060018216600081146122d657600181146122e85761231b565b60ff198316865260208601935061231b565b6122f18561228f565b60005b83811015612313578154818901526001820191506020810190506122f4565b808801955050505b50505092915050565b60006060820190506123396000830186612171565b818103602083015261234b81856122a4565b9050818103604083015261235f818461192d565b9050949350505050565b7f45524332303a2064656372656173656420616c6c6f77616e63652062656c6f7760008201527f207a65726f000000000000000000000000000000000000000000000000000000602082015250565b60006123c56025836118d8565b91506123d082612369565b604082019050919050565b600060208201905081810360008301526123f4816123b8565b9050919050565b7f436f6d6d69746d656e742069732066726f6d2061207369626c696e672070726f60008201527f706f73616c2e0000000000000000000000000000000000000000000000000000602082015250565b60006124576026836118d8565b9150612462826123fb565b604082019050919050565b600060208201905081810360008301526124868161244a565b9050919050565b600061249882611ab8565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8214156124cb576124ca611f5e565b5b600182019050919050565b7f45524332303a20617070726f76652066726f6d20746865207a65726f2061646460008201527f7265737300000000000000000000000000000000000000000000000000000000602082015250565b60006125326024836118d8565b915061253d826124d6565b604082019050919050565b6000602082019050818103600083015261256181612525565b9050919050565b7f45524332303a20617070726f766520746f20746865207a65726f20616464726560008201527f7373000000000000000000000000000000000000000000000000000000000000602082015250565b60006125c46022836118d8565b91506125cf82612568565b604082019050919050565b600060208201905081810360008301526125f3816125b7565b9050919050565b7f45524332303a20696e73756666696369656e7420616c6c6f77616e6365000000600082015250565b6000612630601d836118d8565b915061263b826125fa565b602082019050919050565b6000602082019050818103600083015261265f81612623565b9050919050565b7f45524332303a207472616e736665722066726f6d20746865207a65726f20616460008201527f6472657373000000000000000000000000000000000000000000000000000000602082015250565b60006126c26025836118d8565b91506126cd82612666565b604082019050919050565b600060208201905081810360008301526126f1816126b5565b9050919050565b7f45524332303a207472616e7366657220746f20746865207a65726f206164647260008201527f6573730000000000000000000000000000000000000000000000000000000000602082015250565b60006127546023836118d8565b915061275f826126f8565b604082019050919050565b6000602082019050818103600083015261278381612747565b9050919050565b7f45524332303a207472616e7366657220616d6f756e742065786365656473206260008201527f616c616e63650000000000000000000000000000000000000000000000000000602082015250565b60006127e66026836118d8565b91506127f18261278a565b604082019050919050565b60006020820190508181036000830152612815816127d9565b9050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600061285682611ab8565b9150600082141561286a57612869611f5e565b5b600182039050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603160045260246000fd5b6128ad816119bc565b82525050565b6128bc81611e3f565b82525050565b60006060820190506128d760008301866128a4565b6128e460208301856128b3565b6128f16040830184611c2d565b94935050505056fea2646970667358221220c52150b62ba05136f89366f99f9a8b9757c5999680572ed98111607acf2735d964736f6c63430008090033";

type IdeaConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: IdeaConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class Idea__factory extends ContractFactory {
  constructor(...args: IdeaConstructorParams) {
    if (isSuperArgs(args)) {
      super(...args);
    } else {
      super(_abi, _bytecode, args[0]);
    }
  }

  override deploy(
    _name: PromiseOrValue<string>,
    _symbol: PromiseOrValue<string>,
    _supply: PromiseOrValue<BigNumberish>,
    _ipfsAddr: PromiseOrValue<string>,
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): Promise<Idea> {
    return super.deploy(
      _name,
      _symbol,
      _supply,
      _ipfsAddr,
      overrides || {}
    ) as Promise<Idea>;
  }
  override getDeployTransaction(
    _name: PromiseOrValue<string>,
    _symbol: PromiseOrValue<string>,
    _supply: PromiseOrValue<BigNumberish>,
    _ipfsAddr: PromiseOrValue<string>,
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): TransactionRequest {
    return super.getDeployTransaction(
      _name,
      _symbol,
      _supply,
      _ipfsAddr,
      overrides || {}
    );
  }
  override attach(address: string): Idea {
    return super.attach(address) as Idea;
  }
  override connect(signer: Signer): Idea__factory {
    return super.connect(signer) as Idea__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): IdeaInterface {
    return new utils.Interface(_abi) as IdeaInterface;
  }
  static connect(address: string, signerOrProvider: Signer | Provider): Idea {
    return new Contract(address, _abi, signerOrProvider) as Idea;
  }
}
