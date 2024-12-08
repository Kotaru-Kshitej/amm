import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Amm } from "../target/types/amm";
import { randomBytes } from "crypto"
import { getAssociatedTokenAddress, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token"

describe("amm", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);
  const METADATA_SEED = "metadata"
  const TOKEN_METADATA_PROGRAM_ID = new web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
  const MINT_SEED = "mint"

  const program = anchor.workspace.Amm as Program<Amm>;
  const payer = web3.Keypair.generate()
  const metadata = {
    name: "Just a Test Token",
    symbol: "TEST",
    uri: "https://arweave.net/7UtxcnH13Y1uBCwCnkL6APKsge0hAgacQFl-zFW9NlI",
    decimals: 1,
  }
  const timestamp = new anchor.BN(100)
  const [mint] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from(MINT_SEED),
    provider.wallet.publicKey.toBuffer(),
    timestamp.toArrayLike(Buffer, "le", 8)
    ],
    program.programId
  );
  const [ammPubkey] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("amm"), mint.toBuffer(), timestamp.toArrayLike(Buffer, "le", 8)],
    program.programId
  )
  const [adminPubkey] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("admin")],
    program.programId
  )

  const [metadataAddress] = web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_SEED),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );
  it("fund all the wallets", async () => {
    // let tx = await provider.connection.requestAirdrop(payer.publicKey, web3.LAMPORTS_PER_SOL * 100)
    // await provider.connection.confirmTransaction(tx)
    console.log("mint: ", mint.toBase58())
    console.log("amm: ", ammPubkey.toBase58())

    const eventListener = program.addEventListener("tradeEvent", (e, slot) => {
      console.log("Trade event occured at slot: ", slot)
      console.log(e)
    })
    const ammeventListener = program.addEventListener("ammInitalized", (e, slot) => {
      console.log("AMM INIT occured at slot: ", slot)
      console.log(e)
    })
  })
  it("Admin initalized", async () => {
    const adminData = await program.account.admin.fetch(adminPubkey)
    if (!adminData) {
      console.log("Admin already exists")
      return;
    }
    const tx = await program.methods.initializeAdmin()
      .accountsPartial({
        signer: provider.wallet.publicKey,
        admin: adminPubkey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc()
    console.log("Admin set succefully: ", adminPubkey.toBase58())
  })
  it("Amm initalized", async () => {

    try {

      const ammData = await program.account.amm.fetch(ammPubkey)
      if (ammData.ammBump) {
        console.log("AMM already exists")
        return;
      }
    } catch (e) {

      const tx = await program.methods.initializeAmm({
        name: metadata.name,
        mintCap: new anchor.BN(999912),
        solCap: new anchor.BN(1200),
        symbol: metadata.symbol,
        uri: metadata.uri,
        decimals: metadata.decimals,
        seed: timestamp
      })
        .accountsPartial({
          signer: provider.wallet.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          metadata: metadataAddress,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          admin: adminPubkey,
          amm: ammPubkey,
          mint: mint
        })
        // .signers([payer])
        .rpc()
      console.log(ammPubkey.toBase58())
      const data = await program.account.amm.fetch(ammPubkey)
      console.log("AMM :", ammPubkey)
    }
  });
  it("can buy memecoin", async () => {
    const ammData = await program.account.amm.fetch(ammPubkey)


    const tx = await program.methods.buyCoin(new anchor.BN(web3.LAMPORTS_PER_SOL))
      .accountsPartial({
        signer: provider.wallet.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        mint,
        creator: ammData.creator,
        amm: ammPubkey,
      })
      // .signers([payer])
      .rpc()
    await new Promise((resolve) => setTimeout(resolve, 5000));
    console.log(tx)
  })
  it("can sell memecoin", async () => {
    const ammData = await program.account.amm.fetch(ammPubkey)

    // const tokenAccount = await getAssociatedTokenAddressSync(mint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID)
    // const balace = (await provider.connection.getTokenAccountBalance(tokenAccount)).value.amount
    const tx = await program.methods.sellCoin(new anchor.BN(100))
      .accountsPartial({
        signer: provider.wallet.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        mint,
        creator: ammData.creator,
        amm: ammPubkey
      })
      // .signers([payer])
      .rpc()
    console.log(tx)
  })
  it("can claim fee", async () => {
    const signature = await program.methods.claimFee()
      .accounts({
        signer: provider.wallet.publicKey
      })
      .rpc()
    console.log("claimed fee: ", signature)
  })
  it("close all event listeners", async () => {
    await program.removeEventListener(0)
    await program.removeEventListener(1)
  })
});
