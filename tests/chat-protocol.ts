import * as anchor from "@coral-xyz/anchor"
import { Program, BN } from "@coral-xyz/anchor"
import { Chat } from "../target/types/chat"
import { assert } from "chai"
import { utf8 } from "@coral-xyz/anchor/dist/cjs/utils/bytes"
import * as fs from 'fs'
import bs58 from 'bs58'
import { PublicKey, Keypair, Transaction } from '@solana/web3.js' // Import the Keypair class
import { Token, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token"

describe("Chat_Protocol", () => 
{
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.local())

  const program = anchor.workspace.Chat as Program<Chat>
  const publicKey = anchor.AnchorProvider.local().wallet.publicKey
  var usdcMint = undefined
  const usdcTokenDecimalAmount = 6

  const textWith444Characters = "Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla consequat massa quis enim. Donec pede justo, fringilla vel, aliquet nec, vulputate eget, arcu. In enim justo, rhoncus ut, imperdiet a, venenatis vitae, justo. Nullam dictum feli"
  const textWith144Characters = "Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque penatibus et magnis"
  const textWith32Characters = "Lorem ipsum dolor sit amet, cons"
  const textWith28Characters = "Lorem ipsum dolor sit amet,."
  const textWith27Characters = "Lorem ipsum dolor sit amet,"
  const textWith26Characters = "Lorem ipsum dolor sit amet"

  //const m4aCommentSectionNamePrefix = "M4A_" + textWith28Characters //Seed String Can't be more than 32 characters
  const m4aCommentSectionNamePrefix = "M4A"
  //const pliCommentSectionNamePrefix = "PLI_" + textWith28Characters
  const pliCommentSectionNamePrefix = "PLI"
  //const aboutCommentSectionNamePrefix = "About_" + textWith26Characters
  const aboutCommentSectionNamePrefix = "About"
  //const loCommentSectionNamePrefix = "LO_" + textWith27Characters
  const loCommentSectionNamePrefix = "LO"
  //const commentSectionName = textWith32Characters
  const commentSectionName = "Overview"
  const userName = textWith144Characters 
  const comment = textWith444Characters
  const reply = textWith444Characters
  const voteAmount = 400
  const negativeVoteAmount = -400
  const updatedIdea = "Edited Idea"
  const videoDownVote = false
  const postDownVote = false
  const unStar = false
  const unFED = false

  let successorWallet = anchor.web3.Keypair.generate()

  //Load the keypair from config file
  const keypairPath = '/home/fdr1/.config/solana/id.json';
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
  const testingWalletKeypair = Keypair.fromSecretKey(Uint8Array.from(keypairData))

  it("Creates Token Mint For Fees", async () => 
  {
    //Create a new USDC Mint for testing
    usdcMint = await Token.createMint
    (
      program.provider.connection,
      testingWalletKeypair, //Payer for the mint creation
      program.provider.publicKey, // Mint authority (who can mint tokens)
      null, //Freeze authority (optional)
      6, //Decimals for USDC
      TOKEN_PROGRAM_ID //SPL Token program ID
    )

    const walletATA = await deriveWalletATA(program.provider.publicKey, usdcMint.publicKey)
    await createATAForWallet(testingWalletKeypair, usdcMint.publicKey, walletATA)
    await mintUSDCToWallet(usdcMint.publicKey, walletATA)
  })

  it("Initializes Chat Protocol CEO Account", async () => 
  {
    //const keypair = Keypair.fromSecretKey(bs58.decode("Private Key String")); 
    //console.log(keypair.secretKey) //prints out U8Int array to put in .config/solana/id.json file if you want to put in your own wallet.
    //Will need to delete target folder, run "cargo clean" cmd, "solana air drop 100 sol <pulicAddressString>" to wallet to have enough to deploy, then build and deploy

    await program.methods.initializeChatProtocolAdminAccounts().rpc()
    
    var ceoAccount = await program.account.chatProtocolCeo.fetch(getChatProtocolCEOAccountPDA())
    assert(ceoAccount.address.toBase58() == program.provider.publicKey.toBase58())
  })

  it("Passes on the Chat Protocol CEO Account", async () => 
  {
    await airDropSol(successorWallet.publicKey)

    await program.methods.passOnChatProtocolCeo(successorWallet.publicKey, ).rpc()
    
    var ceoAccount = await program.account.chatProtocolCeo.fetch(getChatProtocolCEOAccountPDA())
    assert(ceoAccount.address.toBase58() == successorWallet.publicKey.toBase58())
  })
  
  it("Passes back the Chat Protocol CEO Account", async () => 
  {
    await program.methods.passOnChatProtocolCeo(program.provider.publicKey, ).
    accounts({signer: successorWallet.publicKey})
    .signers([successorWallet])
    .rpc()
    
    var ceoAccount = await program.account.chatProtocolCeo.fetch(getChatProtocolCEOAccountPDA())
    assert(ceoAccount.address.toBase58() == program.provider.publicKey.toBase58())
  })

  it("Initializes Quailty of Life Accounts", async () => 
  {
    await program.methods.initializeQualityOfLifeAccounts().rpc()
  })

  it("Adds a Fee Token Entry Then Removes It", async () => 
  {
    await program.methods.addFeeTokenEntry(usdcMint.publicKey, usdcTokenDecimalAmount).rpc()
    await program.methods.removeFeeTokenEntry(usdcMint.publicKey).rpc()
  })

  it("Adds a Fee Token Entry", async () => 
  {
    await program.methods.addFeeTokenEntry(usdcMint.publicKey, usdcTokenDecimalAmount).rpc()
  })

  it("Initializes Chat Protocol", async () => 
  {
    await program.methods.initializeChatProtocol().rpc()
  })

  it("Creates User Chat Account", async () => 
  {
    await program.methods.createChatAccount().rpc()
  })

  it("Updates User Name", async () => 
  {
    await program.methods.updateUserName(usdcMint.publicKey, userName).rpc()
  })

  it("Set Use Custom Name Flag False", async () => 
  {
    await program.methods.setUseCustomNameFlag(usdcMint.publicKey, false).rpc()
  })

  it("Set Use Custom Name Flag True", async () => 
  {
    await program.methods.setUseCustomNameFlag(usdcMint.publicKey, true).rpc()
  })

  it("Creates Poll & Poll Option, Edits Poll & Poll Option, Votes On Poll Option, And Then Toggles The Poll Option and Poll Active Flags", async () => 
  {
    //Create poll and poll option
    await program.methods.createPoll(textWith144Characters).rpc()
    await program.methods.createPollOption(new anchor.BN(0), textWith144Characters).rpc()

    var poll = await program.account.poll.fetch(getPollPDA(0))
    var pollOption = await program.account.pollOption.fetch(getPollOptionPDA(0, 0))
    
    assert(poll.pollName == textWith144Characters)
    assert(pollOption.pollOptionName == textWith144Characters)

    //Edit poll and poll option
    await program.methods.editPoll(new anchor.BN(0), "edited test poll").rpc()
    await program.methods.editPollOption(new anchor.BN(0), 0, "edited test poll option").rpc()

    poll = await program.account.poll.fetch(getPollPDA(0))
    pollOption = await program.account.pollOption.fetch(getPollOptionPDA(0, 0))
    
    assert(poll.pollName == "edited test poll")
    assert(pollOption.pollOptionName == "edited test poll option")

    //Vote poll option
    await program.methods.votePollOption(new anchor.BN(0), 0, usdcMint.publicKey, new anchor.BN(100)).rpc()

    pollOption = await program.account.pollOption.fetch(getPollOptionPDA(0, 0))
    assert(pollOption.upVoteScore.eq(new anchor.BN(100)))
    assert(pollOption.upVoteCount.eq(new anchor.BN(1)))
    assert(pollOption.downVoteScore.eq(new anchor.BN(0)))
    assert(pollOption.downVoteCount.eq(new anchor.BN(0)))

    //Toggle poll and poll option is active flag
    assert(poll.isActive)
    assert(pollOption.isActive)

    await program.methods.setPollFlag(new anchor.BN(0), false).rpc()
    await program.methods.setPollOptionFlag(new anchor.BN(0), 0, false).rpc()
    poll = await program.account.poll.fetch(getPollPDA(0))
    pollOption = await program.account.pollOption.fetch(getPollOptionPDA(0, 0))
    assert(!poll.isActive)
    assert(!pollOption.isActive)

    await program.methods.setPollFlag(new anchor.BN(0), true).rpc()
    await program.methods.setPollOptionFlag(new anchor.BN(0), 0, true).rpc()
    poll = await program.account.poll.fetch(getPollPDA(0))
    pollOption = await program.account.pollOption.fetch(getPollOptionPDA(0, 0))
    assert(poll.isActive)
    assert(pollOption.isActive)
  })

  it("Initializes M4A Chat", async () => 
  {
    await program.methods.initializeM4AChat().rpc()
  })

  it("Creates A M4A Comment Section", async () => 
  {
    await program.methods.createCommentSection(m4aCommentSectionNamePrefix, commentSectionName).rpc()
  })

  it("Set Comment Section Disabled Flag True", async () => 
  {
    await program.methods.setCommentSectionFlag(m4aCommentSectionNamePrefix, commentSectionName, true).rpc()

    const commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(m4aCommentSectionNamePrefix, commentSectionName))

    assert(commentSection.isDisabled == true)
  })
  
  it("Set Comment Section Disabled Flag False", async () => 
  {
    await program.methods.setCommentSectionFlag(m4aCommentSectionNamePrefix, commentSectionName, false).rpc()

    const commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(m4aCommentSectionNamePrefix, commentSectionName))

    assert(commentSection.isDisabled == false)
  })

  it("Vote For M4A Comment Section Video/Page", async () => 
  {
    //Up Vote Comment Section Video/Page
    await program.methods.commentSectionVote
    (
      m4aCommentSectionNamePrefix, commentSectionName,
      usdcMint.publicKey, 
      new anchor.BN(voteAmount)
    ).rpc()

    var commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(m4aCommentSectionNamePrefix, commentSectionName))

    assert(commentSection.videoUpVoteScore.eq(new anchor.BN(voteAmount)))
    assert(commentSection.videoUpVoteCount.eq(new anchor.BN(1)))
    assert(commentSection.videoDownVoteScore.eq(new anchor.BN(0)))
    assert(commentSection.videoDownVoteCount.eq(new anchor.BN(0)))

    //Down Vote Comment Section Video/Page
    if(videoDownVote)
    {
      await program.methods.commentSectionVote
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        usdcMint.publicKey, 
        new anchor.BN(negativeVoteAmount)
      ).rpc()

      commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(m4aCommentSectionNamePrefix, commentSectionName))

      assert(commentSection.videoUpVoteScore.eq(new anchor.BN(voteAmount)))
      assert(commentSection.videoUpVoteCount.eq(new anchor.BN(1)))
      assert(commentSection.videoDownVoteScore.eq(new anchor.BN(voteAmount)))
      assert(commentSection.videoDownVoteCount.eq(new anchor.BN(1)))
    }
  })

  it("Posts A M4A Comment, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Then Deletes M4A Comments", async () => 
  {
    //Post 100 Comments
    for(var i=1; i<=1; i++)
    { 
      console.log("M4A Comment: ", i)

      //Post Comment
      await program.methods.postM4AComment
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        usdcMint.publicKey, 
        comment
      ).rpc()

      var m4aComments = await program.account.m4AComment.all()
      
      var newM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      //console.log("Comment: ", newM4AComment[0].account)
      
      assert(newM4AComment[0].account.id.eq(new anchor.BN(i)))

      //Edit Comment
      const editedText = "Edited Comment"

      await program.methods.editM4AComment
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4AComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        editedText
      ).rpc()

      m4aComments = await program.account.m4AComment.all()

      var editedM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(editedM4AComment[0].account.msg == editedText)

      //Up Vote Comment
      await program.methods.m4ACommentVote
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4AComment[0].account.postOwnerAddress,
        newM4AComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      m4aComments = await program.account.m4AComment.all()

      var upVotedM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(upVotedM4AComment[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Comment
      if(postDownVote)
      {
        await program.methods.m4ACommentVote
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4AComment[0].account.postOwnerAddress,
          newM4AComment[0].account.chatAccountPostCountIndex,
          usdcMint.publicKey, 
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        m4aComments = await program.account.m4AComment.all()

        var downVotedM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))
        
        assert(downVotedM4AComment[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Comment
      await program.methods.starM4AComment
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4AComment[0].account.postOwnerAddress,
        newM4AComment[0].account.chatAccountPostCountIndex
      ).rpc()

      m4aComments = await program.account.m4AComment.all()

      var starredM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

      assert(starredM4AComment[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4AComment[0].account.postOwnerAddress,
        newM4AComment[0].account.chatAccountPostCountIndex,
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedM4ACommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newM4AComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedM4ACommentIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4AComment[0].account.postOwnerAddress,
        newM4AComment[0].account.chatAccountPostCountIndex,
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedM4ACommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newM4AComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedM4ACommentIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4AComment[0].account.postOwnerAddress,
        newM4AComment[0].account.chatAccountPostCountIndex,
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedM4ACommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newM4AComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedM4ACommentIdea[0].account.isUpdated == true)
      assert(updatedM4ACommentIdea[0].account.idea == updatedIdea)

      //Unstar Comment
      if(unStar)
      {
        await program.methods.unstarM4AComment
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4AComment[0].account.postOwnerAddress,
          newM4AComment[0].account.chatAccountPostCountIndex
        ).rpc()

        m4aComments = await program.account.m4AComment.all()

        var starredM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

        assert(starredM4AComment[0].account.isStarred == false)
      }

      //FED Comment
      await program.methods.fedM4AComment
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4AComment[0].account.postOwnerAddress,
        newM4AComment[0].account.chatAccountPostCountIndex
      ).rpc()

      m4aComments = await program.account.m4AComment.all()

      var fedM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

      assert(fedM4AComment[0].account.isFed == true)

      //UnFED Comment
      if(unFED)
      {
        await program.methods.unfedM4AComment
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4AComment[0].account.postOwnerAddress,
          newM4AComment[0].account.chatAccountPostCountIndex
        ).rpc()

        m4aComments = await program.account.m4AComment.all()

        var fedM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

        assert(fedM4AComment[0].account.isFed == false)
      }

      //Delete Comment
      await program.methods.deleteM4AComment
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4AComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey
      ).rpc()

      m4aComments = await program.account.m4AComment.all()

      var deletedM4AComment = m4aComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(deletedM4AComment[0].account.isDeleted == true)
    }
  })

  it("Posts A M4A Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes M4A Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("M4A Reply: ", i)

      var m4aComments= await program.account.m4AComment.all()

      //Reply To Comment
      await program.methods.replyToM4AComment
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aComments[0].account.postOwnerAddress,
        m4aComments[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var m4aReplies = await program.account.m4AReply.all()

      var chatAccountReply = m4aReplies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountReply[0].account.msg == reply)

      const newM4AReply = chatAccountReply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editM4AReply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      m4aReplies = await program.account.m4AReply.all()

      var editedM4AReply = m4aReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4AReply.id))

      assert(editedM4AReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.m4AReplyVote
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      m4aReplies = await program.account.m4AReply.all()

      var upVotedM4AReply = m4aReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4AReply.id))

      assert(upVotedM4AReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.m4AReplyVote
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          m4aReplies[0].account.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        m4aReplies = await program.account.m4AReply.all()

        var downVotedM4AReply = m4aReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4AReply.id))

        assert(downVotedM4AReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starM4AReply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      m4aReplies = await program.account.m4AReply.all()

      var starredM4AReply = m4aReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4AReply.id))

      assert(starredM4AReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedM4AReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedM4AReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedM4AReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedM4AReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedM4AReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedM4AReplyIdea[0].account.isUpdated == true)
      assert(updatedM4AReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarM4AReply
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          m4aReplies[0].account.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()
        

        m4aReplies = await program.account.m4AReply.all()

        var starredM4AReply = m4aReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4AReply.id))

        assert(starredM4AReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedM4AReply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      m4aReplies = await program.account.m4AReply.all()

      var fedM4AReply = m4aReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4AReply.id))

      assert(fedM4AReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedM4AReply
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          m4aReplies[0].account.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        m4aReplies = await program.account.m4AReply.all()

        var fedM4AReply = m4aReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4AReply.id))

        assert(fedM4AReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteM4AReply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
      ).rpc()

      m4aReplies = await program.account.m4AReply.all()

      var deletedM4AReply = m4aReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4AReply.id))

      assert(deletedM4AReply[0].account.isDeleted == true)
    }
  })

  it("Posts A M4A Reply To Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes M4A Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("M4A Reply To Reply: ", i)

      var m4aReplies= await program.account.m4AReply.all()

      //Reply To Reply
      await program.methods.replyToM4AReply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aReplies[0].account.postOwnerAddress,
        m4aReplies[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var m4aLv3Replies = await program.account.m4ALv3Reply.all()

      var chatAccountLv3Reply = m4aLv3Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountLv3Reply[0].account.msg == reply)

      const newM4ALv3Reply = chatAccountLv3Reply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editM4ALv3Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      m4aLv3Replies = await program.account.m4ALv3Reply.all()

      var editedM4AReply = m4aLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv3Reply.id))

      assert(editedM4AReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.m4ALv3ReplyVote
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      m4aLv3Replies = await program.account.m4ALv3Reply.all()

      var upVotedM4AReply = m4aLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv3Reply.id))

      assert(upVotedM4AReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.m4ALv3ReplyVote
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4ALv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        m4aLv3Replies = await program.account.m4ALv3Reply.all()

        var downVotedM4AReply = m4aLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv3Reply.id))

        assert(downVotedM4AReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starM4ALv3Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      m4aLv3Replies = await program.account.m4ALv3Reply.all()

      var starredM4AReply = m4aLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv3Reply.id))

      assert(starredM4AReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedM4ALv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedM4ALv3ReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedM4ALv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedM4ALv3ReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedM4ALv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedM4ALv3ReplyIdea[0].account.isUpdated == true)
      assert(updatedM4ALv3ReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarM4ALv3Reply
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4ALv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        ).rpc()

        m4aLv3Replies = await program.account.m4ALv3Reply.all()

        var starredM4AReply = m4aLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv3Reply.id))

        assert(starredM4AReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedM4ALv3Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      m4aLv3Replies = await program.account.m4ALv3Reply.all()

      var fedM4AReply = m4aLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv3Reply.id))

      assert(fedM4AReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedM4ALv3Reply
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4ALv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        m4aLv3Replies = await program.account.m4ALv3Reply.all()

        var fedM4AReply = m4aLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv3Reply.id))

        assert(fedM4AReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteM4ALv3Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      m4aLv3Replies = await program.account.m4ALv3Reply.all()

      var deletedM4AReply = m4aLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv3Reply.id))

      assert(deletedM4AReply[0].account.isDeleted == true)
    }
  })

  it("Posts A M4A Reply To Reply To Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes, Then Replies To M4A Reply To Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("M4A Reply To Reply To Reply: ", i)

      var m4aLv3Replies = await program.account.m4ALv3Reply.all()

      //Reply To Reply
      await program.methods.replyToM4ALv3Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        m4aLv3Replies[0].account.postOwnerAddress,
        m4aLv3Replies[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var m4aLv4Replies = await program.account.m4ALv4Reply.all()

      var chatAccountLv4Reply = m4aLv4Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountLv4Reply[0].account.msg == reply)

      const newM4ALv4Reply = chatAccountLv4Reply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editM4ALv4Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      m4aLv4Replies = await program.account.m4ALv4Reply.all()

      var editedM4AReply = m4aLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv4Reply.id))

      assert(editedM4AReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.m4ALv4ReplyVote(
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)).rpc()

      m4aLv4Replies = await program.account.m4ALv4Reply.all()

      var upVotedM4AReply = m4aLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv4Reply.id))

      assert(upVotedM4AReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.m4ALv4ReplyVote
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4ALv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        m4aLv4Replies = await program.account.m4ALv4Reply.all()

        var downVotedM4AReply = m4aLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv4Reply.id))

        assert(downVotedM4AReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starM4ALv4Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      m4aLv4Replies = await program.account.m4ALv4Reply.all()

      var starredM4AReply = m4aLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv4Reply.id))

      assert(starredM4AReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedM4ALv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedM4ALv4ReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedM4ALv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedM4ALv4ReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedM4ALv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == m4aCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedM4ALv4ReplyIdea[0].account.isUpdated == true)
      assert(updatedM4ALv4ReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarM4ALv4Reply
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4ALv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        m4aLv4Replies = await program.account.m4ALv4Reply.all()

        var starredM4AReply = m4aLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv4Reply.id))

        assert(starredM4AReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedM4ALv4Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        newM4ALv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      m4aLv4Replies = await program.account.m4ALv4Reply.all()

      var fedM4AReply = m4aLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv4Reply.id))

      assert(fedM4AReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedM4ALv4Reply
        (
          m4aCommentSectionNamePrefix, commentSectionName,
          newM4ALv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        m4aLv4Replies = await program.account.m4ALv4Reply.all()

        var fedM4AReply = m4aLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv4Reply.id))

        assert(fedM4AReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteM4ALv4Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      m4aLv4Replies = await program.account.m4ALv4Reply.all()

      var deletedM4AReply = m4aLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newM4ALv4Reply.id))

      assert(deletedM4AReply[0].account.isDeleted == true)

      //Reply To Reply
      const replyToLv4Reply = "Why you delete reply to reply to reply? :0"

      await program.methods.replyToM4ALv4Reply
      (
        m4aCommentSectionNamePrefix, commentSectionName,
        deletedM4AReply[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        replyToLv4Reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      m4aLv4Replies = await program.account.m4ALv4Reply.all()

      var replyToM4ALv4Reply = m4aLv4Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(replyToM4ALv4Reply[0].account.msg == replyToLv4Reply)
    }
  })

  it("Initializes PLI Chat", async () => 
  {
    await program.methods.initializePliChat().rpc()
  })

  it("Creates A PLI Comment Section", async () => 
  {
    await program.methods.createCommentSection(pliCommentSectionNamePrefix, commentSectionName).rpc()
  })

  it("Vote For PLI Comment Section Video/Page", async () => 
  {
    //Up Vote Comment Section Video/Page
    await program.methods.commentSectionVote
    (
      pliCommentSectionNamePrefix, commentSectionName,
      usdcMint.publicKey,
      new anchor.BN(voteAmount)
    ).rpc()

    var commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(pliCommentSectionNamePrefix, commentSectionName))

    assert(commentSection.videoUpVoteScore.eq(new anchor.BN(voteAmount)))
    assert(commentSection.videoUpVoteCount.eq(new anchor.BN(1)))
    assert(commentSection.videoDownVoteScore.eq(new anchor.BN(0)))
    assert(commentSection.videoDownVoteCount.eq(new anchor.BN(0)))

    //Down Vote Comment Section Video/Page
    if(videoDownVote)
    {
      await program.methods.commentSectionVote
      (
        pliCommentSectionNamePrefix, commentSectionName,
        usdcMint.publicKey,
        new anchor.BN(negativeVoteAmount)
      ).rpc()

      commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(pliCommentSectionNamePrefix, commentSectionName))

      assert(commentSection.videoUpVoteScore.eq(new anchor.BN(voteAmount)))
      assert(commentSection.videoUpVoteCount.eq(new anchor.BN(1)))
      assert(commentSection.videoDownVoteScore.eq(new anchor.BN(voteAmount)))
      assert(commentSection.videoDownVoteCount.eq(new anchor.BN(1)))
    }
  })

  it("Posts A PLI Comment, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Then Deletes PLI Comment", async () => 
  {
    //Post 100 Comments
    for(var i=1; i<=1; i++)
    { 
      console.log("PLI Comment: ", i)

      //Post Comment
      await program.methods.postPliComment
      (
        pliCommentSectionNamePrefix, commentSectionName,
        usdcMint.publicKey,
        comment
      ).rpc()

      var pliComments = await program.account.pliComment.all()
      
      var newPLIComment = pliComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      //console.log("Comment: ", newPLIComment[0].account)
      
      assert(newPLIComment[0].account.id.eq(new anchor.BN(i)))

      //Edit Comment
      const editedText = "Edited Comment"

      await program.methods.editPliComment
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        editedText
      ).rpc()

      pliComments = await program.account.pliComment.all()

      var editedPLIComment = pliComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(editedPLIComment[0].account.msg == editedText)

      //Up Vote Comment
      await program.methods.pliCommentVote
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIComment[0].account.postOwnerAddress,
        newPLIComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      pliComments = await program.account.pliComment.all()

      var upVotedPLIComment = pliComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(upVotedPLIComment[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Comment
      if(postDownVote)
      {
        await program.methods.pliCommentVote
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLIComment[0].account.postOwnerAddress,
          newPLIComment[0].account.chatAccountPostCountIndex,
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()
      
        pliComments = await program.account.pliComment.all()

        var downVotedPLIComment = pliComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

        assert(downVotedPLIComment[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Comment
      await program.methods.starPliComment
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIComment[0].account.postOwnerAddress,
        newPLIComment[0].account.chatAccountPostCountIndex
      ).rpc()

      pliComments = await program.account.pliComment.all()

      var starredPLIComment = pliComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

      assert(starredPLIComment[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIComment[0].account.postOwnerAddress,
        newPLIComment[0].account.chatAccountPostCountIndex,
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newPLIComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedPLICommentIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIComment[0].account.postOwnerAddress,
        newPLIComment[0].account.chatAccountPostCountIndex,
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newPLIComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedPLICommentIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIComment[0].account.postOwnerAddress,
        newPLIComment[0].account.chatAccountPostCountIndex,
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newPLIComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedPLICommentIdea[0].account.isUpdated == true)
      assert(updatedPLICommentIdea[0].account.idea == updatedIdea)

      //Unstar Comment
      if(unStar)
      {
        await program.methods.unstarPliComment
        (
          pliCommentSectionNamePrefix, commentSectionName,
          publicKey,
          newPLIComment[0].account.chatAccountPostCountIndex
        ).rpc()

        pliComments = await program.account.pliComment.all()

        var starredPLIComment = pliComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

        assert(starredPLIComment[0].account.isStarred == false)
      }

      //FED Comment
      await program.methods.fedPliComment
      (
        pliCommentSectionNamePrefix, commentSectionName,
        publicKey,
        newPLIComment[0].account.chatAccountPostCountIndex
      ).rpc()

      pliComments = await program.account.pliComment.all()

      var fedPLIComment = pliComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

      assert(fedPLIComment[0].account.isFed == true)

      //UnFED Comment
      if(unFED)
      {
        await program.methods.unfedPliComment
        (
          pliCommentSectionNamePrefix, commentSectionName,
          publicKey,
          newPLIComment[0].account.chatAccountPostCountIndex
        ).rpc()

        pliComments = await program.account.pliComment.all()

        var fedPLIComment = pliComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

        assert(fedPLIComment[0].account.isFed == false)
      }

      //Delete Comment
      await program.methods.deletePliComment
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
      ).rpc()

      pliComments = await program.account.pliComment.all()

      var deletedPLIComment = pliComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(deletedPLIComment[0].account.isDeleted == true)
    }
  })

  it("Posts A PLI Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes PLI Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("PLI Reply: ", i)

      var pliComments= await program.account.pliComment.all()
      
      //Reply To Comment
      await program.methods.replyToPliComment
      (
        pliCommentSectionNamePrefix, commentSectionName,
        pliComments[0].account.postOwnerAddress,
        pliComments[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var pliReplies = await program.account.pliReply.all()

      var chatAccountReply = pliReplies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountReply[0].account.msg == reply)

      const newPLIReply = chatAccountReply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editPliReply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      pliReplies = await program.account.pliReply.all()

      var editedPLIReply = pliReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLIReply.id))

      assert(editedPLIReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.pliReplyVote
      (
        pliCommentSectionNamePrefix, commentSectionName,
        pliReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      pliReplies = await program.account.pliReply.all()

      var upVotedPLIReply = pliReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLIReply.id))

      assert(upVotedPLIReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.pliReplyVote
        (
          pliCommentSectionNamePrefix, commentSectionName,
          pliReplies[0].account.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()
      
        pliReplies = await program.account.pliReply.all()

        var downVotedPLIReply = pliReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLIReply.id))

        assert(downVotedPLIReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starPliReply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      pliReplies = await program.account.pliReply.all()

      var starredPLIReply = pliReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLIReply.id))

      assert(starredPLIReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedPLIReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedPLIReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedPLIReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedPLIReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedPLIReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedPLIReplyIdea[0].account.isUpdated == true)
      assert(updatedPLIReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarPliReply
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLIReply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        pliReplies = await program.account.pliReply.all()

        var starredPLIReply = pliReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLIReply.id))

        assert(starredPLIReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedPliReply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLIReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      pliReplies = await program.account.pliReply.all()

      var fedPLIReply = pliReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLIReply.id))

      assert(fedPLIReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedPliReply
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLIReply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        pliReplies = await program.account.pliReply.all()

        var fedPLIReply = pliReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLIReply.id))

        assert(fedPLIReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deletePliReply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      pliReplies = await program.account.pliReply.all()

      var deletedPLIReply = pliReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLIReply.id))

      assert(deletedPLIReply[0].account.isDeleted == true)
    }
  })

  it("Posts A PLI Reply To Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes PLI Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("PLI Reply To Reply: ", i)

      var pliReplies= await program.account.pliReply.all()

      //Reply To Reply
      await program.methods.replyToPliReply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        pliReplies[0].account.postOwnerAddress,
        pliReplies[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var pliLv3Replies = await program.account.pliLv3Reply.all()

      var chatAccountLv3Reply = pliLv3Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountLv3Reply[0].account.msg == reply)

      const newPLILv3Reply = chatAccountLv3Reply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editPliLv3Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      pliLv3Replies = await program.account.pliLv3Reply.all()

      var editedPLIReply = pliLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv3Reply.id))

      assert(editedPLIReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.pliLv3ReplyVote
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      pliLv3Replies = await program.account.pliLv3Reply.all()

      var upVotedPLIReply = pliLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv3Reply.id))

      assert(upVotedPLIReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.pliLv3ReplyVote
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLILv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        pliLv3Replies = await program.account.pliLv3Reply.all()

        var downVotedPLIReply = pliLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv3Reply.id))

        assert(downVotedPLIReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starPliLv3Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      pliLv3Replies = await program.account.pliLv3Reply.all()

      var starredPLIReply = pliLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv3Reply.id))

      assert(starredPLIReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedPLILv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedPLILv3ReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedPLILv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedPLILv3ReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedPLILv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedPLILv3ReplyIdea[0].account.isUpdated == true)
      assert(updatedPLILv3ReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarPliLv3Reply
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLILv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        pliLv3Replies = await program.account.pliLv3Reply.all()

        var starredPLIReply = pliLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv3Reply.id))

        assert(starredPLIReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedPliLv3Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      pliLv3Replies = await program.account.pliLv3Reply.all()

      var fedPLIReply = pliLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv3Reply.id))

      assert(fedPLIReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedPliLv3Reply
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLILv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        pliLv3Replies = await program.account.pliLv3Reply.all()

        var fedPLIReply = pliLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv3Reply.id))

        assert(fedPLIReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deletePliLv3Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
      ).rpc()

      pliLv3Replies = await program.account.pliLv3Reply.all()

      var deletedPLIReply = pliLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv3Reply.id))

      assert(deletedPLIReply[0].account.isDeleted == true)
    }
  })

  it("Posts A PLI Reply To Reply To Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes, Then Replies To PLI Reply To Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("PLI Reply To Reply To Reply: ", i)

      var pliLv3Replies = await program.account.pliLv3Reply.all()

      //Reply To Reply
      await program.methods.replyToPliLv3Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        pliLv3Replies[0].account.postOwnerAddress,
        pliLv3Replies[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var pliLv4Replies = await program.account.pliLv4Reply.all()

      var chatAccountLv4Reply = pliLv4Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountLv4Reply[0].account.msg == reply)

      const newPLILv4Reply = chatAccountLv4Reply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editPliLv4Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      pliLv4Replies = await program.account.pliLv4Reply.all()

      var editedPLIReply = pliLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv4Reply.id))

      assert(editedPLIReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.pliLv4ReplyVote
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      pliLv4Replies = await program.account.pliLv4Reply.all()

      var upVotedPLIReply = pliLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv4Reply.id))

      assert(upVotedPLIReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.pliLv4ReplyVote
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLILv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        pliLv4Replies = await program.account.pliLv4Reply.all()

        var downVotedPLIReply = pliLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv4Reply.id))

        assert(downVotedPLIReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starPliLv4Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      pliLv4Replies = await program.account.pliLv4Reply.all()

      var starredPLIReply = pliLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv4Reply.id))

      assert(starredPLIReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedPLILv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedPLILv4ReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedPLILv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedPLILv4ReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedPLILv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == pliCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedPLILv4ReplyIdea[0].account.isUpdated == true)
      assert(updatedPLILv4ReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarPliLv4Reply
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLILv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        pliLv4Replies = await program.account.pliLv4Reply.all()

        var starredPLIReply = pliLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv4Reply.id))

        assert(starredPLIReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedPliLv4Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        newPLILv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      pliLv4Replies = await program.account.pliLv4Reply.all()

      var fedPLIReply = pliLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv4Reply.id))

      assert(fedPLIReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedPliLv4Reply
        (
          pliCommentSectionNamePrefix, commentSectionName,
          newPLILv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        pliLv4Replies = await program.account.pliLv4Reply.all()

        var fedPLIReply = pliLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv4Reply.id))

        assert(fedPLIReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deletePliLv4Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      pliLv4Replies = await program.account.pliLv4Reply.all()

      var deletedPLIReply = pliLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newPLILv4Reply.id))

      assert(deletedPLIReply[0].account.isDeleted == true)

      //Reply To Reply
      const replyToLv4Reply = "Why you delete reply to reply to reply? :0"

      await program.methods.replyToPliLv4Reply
      (
        pliCommentSectionNamePrefix, commentSectionName,
        deletedPLIReply[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        replyToLv4Reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      pliLv4Replies = await program.account.pliLv4Reply.all()

      var replyToPLILv4Reply = pliLv4Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(replyToPLILv4Reply[0].account.msg == replyToLv4Reply)
    }
  })

  it("Initializes About Chat", async () => 
  {
    await program.methods.initializeAboutChat().rpc()
  })

  it("Creates An About Comment Section", async () => 
  {
    await program.methods.createCommentSection(aboutCommentSectionNamePrefix, commentSectionName).rpc()
  })

  it("Vote For About Comment Section Video/Page", async () => 
  {
    //Up Vote Comment Section Video/Page
    await program.methods.commentSectionVote
    (
      aboutCommentSectionNamePrefix, commentSectionName,
      usdcMint.publicKey,
      new anchor.BN(voteAmount)
    ).rpc()

    var commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(aboutCommentSectionNamePrefix, commentSectionName))

    assert(commentSection.videoUpVoteScore.eq(new anchor.BN(voteAmount)))
    assert(commentSection.videoUpVoteCount.eq(new anchor.BN(1)))
    assert(commentSection.videoDownVoteScore.eq(new anchor.BN(0)))
    assert(commentSection.videoDownVoteCount.eq(new anchor.BN(0)))

    //Down Vote Comment Section Video/Page
    if(videoDownVote)
    {
      await program.methods.commentSectionVote
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        usdcMint.publicKey,
        new anchor.BN(negativeVoteAmount)
      ).rpc()

      commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(aboutCommentSectionNamePrefix, commentSectionName))

      assert(commentSection.videoUpVoteScore.eq(new anchor.BN(voteAmount)))
      assert(commentSection.videoUpVoteCount.eq(new anchor.BN(1)))
      assert(commentSection.videoDownVoteScore.eq(new anchor.BN(voteAmount)))
      assert(commentSection.videoDownVoteCount.eq(new anchor.BN(1)))
    }
  })

  it("Posts An About Comment, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Then Deletes About Comments", async () => 
  {
    //Post 100 Comments
    for(var i=1; i<=1; i++)
    { 
      console.log("About Comment: ", i)

      //Post Comment
      await program.methods.postAboutComment
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        usdcMint.publicKey,
        comment
      ).rpc()

      var aboutComments = await program.account.aboutComment.all()
      
      var newAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      //console.log("Comment: ", newAboutComment[0].account)
      
      assert(newAboutComment[0].account.id.eq(new anchor.BN(i)))

      //Edit Comment
      const editedText = "Edited Comment"

      await program.methods.editAboutComment
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        editedText
      ).rpc()

      aboutComments = await program.account.aboutComment.all()

      var editedAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(editedAboutComment[0].account.msg == editedText)

      //Up Vote Comment
      await program.methods.aboutCommentVote
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutComment[0].account.postOwnerAddress,
        newAboutComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      aboutComments = await program.account.aboutComment.all()

      var upVotedAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(upVotedAboutComment[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Comment
      if(postDownVote)
      {
        await program.methods.aboutCommentVote
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutComment[0].account.postOwnerAddress,
          newAboutComment[0].account.chatAccountPostCountIndex,
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()
      
        aboutComments = await program.account.aboutComment.all()

        var downVotedAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

        assert(downVotedAboutComment[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Comment
      await program.methods.starAboutComment
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutComment[0].account.postOwnerAddress,
        newAboutComment[0].account.chatAccountPostCountIndex
      ).rpc()

      aboutComments = await program.account.aboutComment.all()

      var starredAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

      assert(starredAboutComment[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutComment[0].account.postOwnerAddress,
        newAboutComment[0].account.chatAccountPostCountIndex,
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newAboutComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedPLICommentIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutComment[0].account.postOwnerAddress,
        newAboutComment[0].account.chatAccountPostCountIndex,
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newAboutComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedPLICommentIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutComment[0].account.postOwnerAddress,
        newAboutComment[0].account.chatAccountPostCountIndex,
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newAboutComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedPLICommentIdea[0].account.isUpdated == true)
      assert(updatedPLICommentIdea[0].account.idea == updatedIdea)

      //Unstar Comment
      if(unStar)
      {
        await program.methods.unstarAboutComment
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutComment[0].account.postOwnerAddress,
          newAboutComment[0].account.chatAccountPostCountIndex
        ).rpc()

        aboutComments = await program.account.aboutComment.all()

        var starredAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

        assert(starredAboutComment[0].account.isStarred == false)
      }

      //FED Comment
      await program.methods.fedAboutComment
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutComment[0].account.postOwnerAddress,
        newAboutComment[0].account.chatAccountPostCountIndex
      ).rpc()

      aboutComments = await program.account.aboutComment.all()

      var fedAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

      assert(fedAboutComment[0].account.isFed == true)

      //UnFED Comment
      if(unFED)
      {
        await program.methods.unfedAboutComment
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutComment[0].account.postOwnerAddress,
          newAboutComment[0].account.chatAccountPostCountIndex
        ).rpc()

        aboutComments = await program.account.aboutComment.all()

        var fedAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

        assert(fedAboutComment[0].account.isFed == false)
      }

      //Delete Comment
      await program.methods.deleteAboutComment
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey
      ).rpc()

      aboutComments = await program.account.aboutComment.all()

      var deletedAboutComment = aboutComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(deletedAboutComment[0].account.isDeleted == true)
    }
  })

  it("Posts An About Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, Deletes, And Then Replies To Replies", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("About Reply: ", i)

      var aboutComments= await program.account.aboutComment.all()
      
      //Reply To Comment
      await program.methods.replyToAboutComment
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        aboutComments[0].account.postOwnerAddress,
        aboutComments[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var aboutReplies = await program.account.aboutReply.all()

      var chatAccountReply = aboutReplies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountReply[0].account.msg == reply)

      const newAboutReply = chatAccountReply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editAboutReply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      aboutReplies = await program.account.aboutReply.all()

      var editedAboutReply = aboutReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutReply.id))

      assert(editedAboutReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.aboutReplyVote
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        aboutReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      aboutReplies = await program.account.aboutReply.all()

      var upVotedAboutReply = aboutReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutReply.id))

      assert(upVotedAboutReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.aboutReplyVote
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          aboutReplies[0].account.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()
      
        aboutReplies = await program.account.aboutReply.all()

        var downVotedAboutReply = aboutReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutReply.id))

        assert(downVotedAboutReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starAboutReply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      aboutReplies = await program.account.aboutReply.all()

      var starredAboutReply = aboutReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutReply.id))

      assert(starredAboutReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedAboutReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedAboutReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedAboutReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedAboutReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedAboutReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedAboutReplyIdea[0].account.isUpdated == true)
      assert(updatedAboutReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarAboutReply
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutReply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        aboutReplies = await program.account.aboutReply.all()

        var starredAboutReply = aboutReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutReply.id))

        assert(starredAboutReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedAboutReply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      aboutReplies = await program.account.aboutReply.all()

      var fedAboutReply = aboutReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutReply.id))

      assert(fedAboutReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedAboutReply
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutReply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        aboutReplies = await program.account.aboutReply.all()

        var fedAboutReply = aboutReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutReply.id))

        assert(fedAboutReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteAboutReply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      aboutReplies = await program.account.aboutReply.all()

      var deletedAboutReply = aboutReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutReply.id))

      assert(deletedAboutReply[0].account.isDeleted == true)
    }
  })

  it("Posts An About Reply To Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes About Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("About Reply To Reply: ", i)

      var aboutReplies= await program.account.aboutReply.all()

      //Reply To Reply
      await program.methods.replyToAboutReply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        aboutReplies[0].account.postOwnerAddress,
        aboutReplies[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var aboutLv3Replies = await program.account.aboutLv3Reply.all()

      var chatAccountLv3Reply = aboutLv3Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountLv3Reply[0].account.msg == reply)

      const newAboutLv3Reply = chatAccountLv3Reply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editAboutLv3Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      aboutLv3Replies = await program.account.aboutLv3Reply.all()

      var editedAboutReply = aboutLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv3Reply.id))

      assert(editedAboutReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.aboutLv3ReplyVote(
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)).rpc()

      aboutLv3Replies = await program.account.aboutLv3Reply.all()

      var upVotedAboutReply = aboutLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv3Reply.id))

      assert(upVotedAboutReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.aboutLv3ReplyVote
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutLv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        aboutLv3Replies = await program.account.aboutLv3Reply.all()

        var downVotedAboutReply = aboutLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv3Reply.id))

        assert(downVotedAboutReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starAboutLv3Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      aboutLv3Replies = await program.account.aboutLv3Reply.all()

      var starredAboutReply = aboutLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv3Reply.id))

      assert(starredAboutReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedAboutLv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedAboutLv3ReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedAboutLv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedAboutLv3ReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedAboutLv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedAboutLv3ReplyIdea[0].account.isUpdated == true)
      assert(updatedAboutLv3ReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarAboutLv3Reply
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutLv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        aboutLv3Replies = await program.account.aboutLv3Reply.all()

        var starredAboutReply = aboutLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv3Reply.id))

        assert(starredAboutReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedAboutLv3Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      aboutLv3Replies = await program.account.aboutLv3Reply.all()

      var fedAboutReply = aboutLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv3Reply.id))

      assert(fedAboutReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedAboutLv3Reply
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutLv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        aboutLv3Replies = await program.account.aboutLv3Reply.all()

        var fedAboutReply = aboutLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv3Reply.id))

        assert(fedAboutReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteAboutLv3Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      aboutLv3Replies = await program.account.aboutLv3Reply.all()

      var deletedAboutReply = aboutLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv3Reply.id))

      assert(deletedAboutReply[0].account.isDeleted == true)
    }
  })

  it("Posts An About Reply To Reply To Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes, Then Replies To About Reply To Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("About Reply To Reply To Reply: ", i)

      var aboutLv3Replies = await program.account.aboutLv3Reply.all()

      //Reply To Reply
      await program.methods.replyToAboutLv3Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        aboutLv3Replies[0].account.postOwnerAddress,
        aboutLv3Replies[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var aboutLv4Replies = await program.account.aboutLv4Reply.all()

      var chatAccountLv4Reply = aboutLv4Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountLv4Reply[0].account.msg == reply)

      const newAboutLv4Reply = chatAccountLv4Reply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editAboutLv4Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      aboutLv4Replies = await program.account.aboutLv4Reply.all()

      var editedAboutReply = aboutLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv4Reply.id))

      assert(editedAboutReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.aboutLv4ReplyVote
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      aboutLv4Replies = await program.account.aboutLv4Reply.all()

      var upVotedAboutReply = aboutLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv4Reply.id))

      assert(upVotedAboutReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.aboutLv4ReplyVote
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutLv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        aboutLv4Replies = await program.account.aboutLv4Reply.all()

        var downVotedAboutReply = aboutLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv4Reply.id))

        assert(downVotedAboutReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starAboutLv4Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      aboutLv4Replies = await program.account.aboutLv4Reply.all()

      var starredAboutReply = aboutLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv4Reply.id))

      assert(starredAboutReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedAboutLv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedAboutLv4ReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedAboutLv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedAboutLv4ReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedAboutLv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == aboutCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedAboutLv4ReplyIdea[0].account.isUpdated == true)
      assert(updatedAboutLv4ReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarAboutLv4Reply
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutLv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        aboutLv4Replies = await program.account.aboutLv4Reply.all()

        var starredAboutReply = aboutLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv4Reply.id))

        assert(starredAboutReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedAboutLv4Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        newAboutLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      aboutLv4Replies = await program.account.aboutLv4Reply.all()

      var fedAboutReply = aboutLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv4Reply.id))

      assert(fedAboutReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedAboutLv4Reply
        (
          aboutCommentSectionNamePrefix, commentSectionName,
          newAboutLv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        aboutLv4Replies = await program.account.aboutLv4Reply.all()

        var fedAboutReply = aboutLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv4Reply.id))

        assert(fedAboutReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteAboutLv4Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      aboutLv4Replies = await program.account.aboutLv4Reply.all()

      var deletedAboutReply = aboutLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newAboutLv4Reply.id))

      assert(deletedAboutReply[0].account.isDeleted == true)

      //Reply To Reply
      const replyToLv4Reply = "Why you delete reply to reply to reply? :0"

      await program.methods.replyToAboutLv4Reply
      (
        aboutCommentSectionNamePrefix, commentSectionName,
        deletedAboutReply[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        replyToLv4Reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      aboutLv4Replies = await program.account.aboutLv4Reply.all()

      var replyToAboutLv4Reply = aboutLv4Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(replyToAboutLv4Reply[0].account.msg == replyToLv4Reply)
    }
  })

  it("Initializes LO Chat", async () => 
  {
    await program.methods.initializeLoChat().rpc()
  })

  it("Creates An LO Comment Section", async () => 
  {
    await program.methods.createCommentSection(loCommentSectionNamePrefix, commentSectionName).rpc()
  })

  it("Vote For About Comment Section Video/Page", async () => 
  {
    //Up Vote Comment Section Video/Page
    await program.methods.commentSectionVote
    (
      loCommentSectionNamePrefix, commentSectionName,
      usdcMint.publicKey,
      new anchor.BN(voteAmount)
    ).rpc()

    var commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(loCommentSectionNamePrefix, commentSectionName))

    assert(commentSection.videoUpVoteScore.eq(new anchor.BN(voteAmount)))
    assert(commentSection.videoUpVoteCount.eq(new anchor.BN(1)))
    assert(commentSection.videoDownVoteScore.eq(new anchor.BN(0)))
    assert(commentSection.videoDownVoteCount.eq(new anchor.BN(0)))

    //Down Vote Comment Section Video/Page
    if(videoDownVote)
    {
      await program.methods.commentSectionVote
      (
        loCommentSectionNamePrefix, commentSectionName,
        usdcMint.publicKey,
        new anchor.BN(negativeVoteAmount)
      ).rpc()

      commentSection = await program.account.commentSection.fetch(getCommentSectionPDA(loCommentSectionNamePrefix, commentSectionName))

      assert(commentSection.videoUpVoteScore.eq(new anchor.BN(voteAmount)))
      assert(commentSection.videoUpVoteCount.eq(new anchor.BN(1)))
      assert(commentSection.videoDownVoteScore.eq(new anchor.BN(voteAmount)))
      assert(commentSection.videoDownVoteCount.eq(new anchor.BN(1)))
    }
  })

  it("Posts An LO Comment, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Then Deletes LO Comments", async () => 
  {
    //Post 100 Comments
    for(var i=1; i<=1; i++)
    { 
      console.log("LO Comment: ", i)

      //Post Comment
      await program.methods.postLoComment
      (
        loCommentSectionNamePrefix, commentSectionName,
        usdcMint.publicKey,
        comment
      ).rpc()

      var loComments = await program.account.loComment.all()
      
      var newLoComment = loComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      //console.log("Comment: ", newLoComment[0].account)
      
      assert(newLoComment[0].account.id.eq(new anchor.BN(i)))

      //Edit Comment
      const editedText = "Edited Comment"

      await program.methods.editLoComment
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        editedText
      ).rpc()

      loComments = await program.account.loComment.all()

      var editedLoComment = loComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(editedLoComment[0].account.msg == editedText)

      //Up Vote Comment
      await program.methods.loCommentVote
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoComment[0].account.postOwnerAddress,
        newLoComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      loComments = await program.account.loComment.all()

      var upVotedLoComment = loComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(upVotedLoComment[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Comment
      if(postDownVote)
      {
        await program.methods.loCommentVote
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoComment[0].account.postOwnerAddress,
          newLoComment[0].account.chatAccountPostCountIndex,
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()
      
        loComments = await program.account.loComment.all()

        var downVotedLoComment = loComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

        assert(downVotedLoComment[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Comment
      await program.methods.starLoComment
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoComment[0].account.postOwnerAddress,
        newLoComment[0].account.chatAccountPostCountIndex
      ).rpc()

      loComments = await program.account.loComment.all()

      var starredLoComment = loComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == loCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

      assert(starredLoComment[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoComment[0].account.postOwnerAddress,
        newLoComment[0].account.chatAccountPostCountIndex,
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newLoComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedPLICommentIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoComment[0].account.postOwnerAddress,
        newLoComment[0].account.chatAccountPostCountIndex,
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newLoComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedPLICommentIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoComment[0].account.postOwnerAddress,
        newLoComment[0].account.chatAccountPostCountIndex,
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedPLICommentIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(newLoComment[0].account.chatAccountPostCountIndex)) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedPLICommentIdea[0].account.isUpdated == true)
      assert(updatedPLICommentIdea[0].account.idea == updatedIdea)

      //Unstar Comment
      if(unStar)
      {
        await program.methods.unstarLoComment
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoComment[0].account.postOwnerAddress,
          newLoComment[0].account.chatAccountPostCountIndex
        ).rpc()

        loComments = await program.account.loComment.all()

        var starredLoComment = loComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == loCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

        assert(starredLoComment[0].account.isStarred == false)
      }

      //FED Comment
      await program.methods.fedLoComment
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoComment[0].account.postOwnerAddress,
        newLoComment[0].account.chatAccountPostCountIndex
      ).rpc()

      loComments = await program.account.loComment.all()

      var fedLoComment = loComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == loCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

      assert(fedLoComment[0].account.isFed == true)

      //UnFED Comment
      if(unFED)
      {
        await program.methods.unfedLoComment
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoComment[0].account.postOwnerAddress,
          newLoComment[0].account.chatAccountPostCountIndex
        ).rpc()

        loComments = await program.account.loComment.all()

        var fedLoComment = loComments.filter((comment: { account: { id: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) => ((comment.account.id.eq(new anchor.BN(i))) && ((comment.account.commentSectionNamePrefix == loCommentSectionNamePrefix) && (comment.account.commentSectionName == commentSectionName))))

        assert(fedLoComment[0].account.isFed == false)
      }

      //Delete Comment
      await program.methods.deleteLoComment
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoComment[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey
      ).rpc()

      loComments = await program.account.loComment.all()

      var deletedLoComment = loComments.filter((comment: { account: { id: anchor.BN }}  ) => comment.account.id.eq(new anchor.BN(i)))

      assert(deletedLoComment[0].account.isDeleted == true)
    }
  })

  it("Posts An LO Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, Deletes, And Then Replies To Replies", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("LO Reply: ", i)

      var loComments= await program.account.loComment.all()
      
      //Reply To Comment
      await program.methods.replyToLoComment
      (
        loCommentSectionNamePrefix, commentSectionName,
        loComments[0].account.postOwnerAddress,
        loComments[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var loReplies = await program.account.loReply.all()

      var chatAccountReply = loReplies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountReply[0].account.msg == reply)

      const newLoReply = chatAccountReply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editLoReply
      (
        loCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      loReplies = await program.account.loReply.all()

      var editedLoReply = loReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoReply.id))

      assert(editedLoReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.loReplyVote
      (
        loCommentSectionNamePrefix, commentSectionName,
        loReplies[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      loReplies = await program.account.loReply.all()

      var upVotedLoReply = loReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoReply.id))

      assert(upVotedLoReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.loReplyVote
        (
          loCommentSectionNamePrefix, commentSectionName,
          loReplies[0].account.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()
      
        loReplies = await program.account.loReply.all()

        var downVotedLoReply = loReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoReply.id))

        assert(downVotedLoReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starLoReply
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      loReplies = await program.account.loReply.all()

      var starredLoReply = loReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoReply.id))

      assert(starredLoReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedLoReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedLoReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedLoReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedLoReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedLoReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedLoReplyIdea[0].account.isUpdated == true)
      assert(updatedLoReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarLoReply
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoReply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        loReplies = await program.account.loReply.all()

        var starredLoReply = loReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoReply.id))

        assert(starredLoReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedLoReply
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoReply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      loReplies = await program.account.loReply.all()

      var fedLoReply = loReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoReply.id))

      assert(fedLoReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedLoReply
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoReply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        loReplies = await program.account.loReply.all()

        var fedLoReply = loReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoReply.id))

        assert(fedLoReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteLoReply
      (
        loCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      loReplies = await program.account.loReply.all()

      var deletedLoReply = loReplies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoReply.id))

      assert(deletedLoReply[0].account.isDeleted == true)
    }
  })

  it("Posts An LO Reply To Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes LO Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("LO Reply To Reply: ", i)

      var loReplies= await program.account.loReply.all()

      //Reply To Reply
      await program.methods.replyToLoReply
      (
        loCommentSectionNamePrefix, commentSectionName,
        loReplies[0].account.postOwnerAddress,
        loReplies[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var loLv3Replies = await program.account.loLv3Reply.all()

      var chatAccountLv3Reply = loLv3Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountLv3Reply[0].account.msg == reply)

      const newLoLv3Reply = chatAccountLv3Reply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editLoLv3Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      loLv3Replies = await program.account.loLv3Reply.all()

      var editedLoReply = loLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv3Reply.id))

      assert(editedLoReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.loLv3ReplyVote(
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)).rpc()

      loLv3Replies = await program.account.loLv3Reply.all()

      var upVotedLoReply = loLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv3Reply.id))

      assert(upVotedLoReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.loLv3ReplyVote
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoLv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        loLv3Replies = await program.account.loLv3Reply.all()

        var downVotedLoReply = loLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv3Reply.id))

        assert(downVotedLoReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starLoLv3Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      loLv3Replies = await program.account.loLv3Reply.all()

      var starredLoReply = loLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv3Reply.id))

      assert(starredLoReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedLoLv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedLoLv3ReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedLoLv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedLoLv3ReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedLoLv3ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedLoLv3ReplyIdea[0].account.isUpdated == true)
      assert(updatedLoLv3ReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarLoLv3Reply
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoLv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        loLv3Replies = await program.account.loLv3Reply.all()

        var starredLoReply = loLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv3Reply.id))

        assert(starredLoReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedLoLv3Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv3Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      loLv3Replies = await program.account.loLv3Reply.all()

      var fedLoReply = loLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv3Reply.id))

      assert(fedLoReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedLoLv3Reply
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoLv3Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        loLv3Replies = await program.account.loLv3Reply.all()

        var fedLoReply = loLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv3Reply.id))

        assert(fedLoReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteLoLv3Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      loLv3Replies = await program.account.loLv3Reply.all()

      var deletedLoReply = loLv3Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv3Reply.id))

      assert(deletedLoReply[0].account.isDeleted == true)
    }
  })

  it("Posts An LO Reply To Reply To Reply, Edits, Up Votes, Down Votes, Stars, Implements Idea, Unimplements Idea, Edits Idea, UnStars, FEDs, UnFEDs, And Deletes, Then Replies To LO Reply To Reply To Reply", async () => 
  {
    //Post 100 Replies
    for(var i=1; i<=1; i++)
    { 
      console.log("LO Reply To Reply To Reply: ", i)

      var loLv3Replies = await program.account.loLv3Reply.all()

      //Reply To Reply
      await program.methods.replyToLoLv3Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        loLv3Replies[0].account.postOwnerAddress,
        loLv3Replies[0].account.chatAccountPostCountIndex,
        usdcMint.publicKey,
        reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      var loLv4Replies = await program.account.loLv4Reply.all()

      var chatAccountLv4Reply = loLv4Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(chatAccountLv4Reply[0].account.msg == reply)

      const newLoLv4Reply = chatAccountLv4Reply[0].account

      //Edit Reply
      const editedText = "Edited Reply"

      await program.methods.editLoLv4Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        editedText
      ).rpc()

      loLv4Replies = await program.account.loLv4Reply.all()

      var editedLoReply = loLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv4Reply.id))

      assert(editedLoReply[0].account.msg == editedText)

      //Up Vote Reply
      await program.methods.loLv4ReplyVote
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        new anchor.BN(voteAmount)
      ).rpc()

      loLv4Replies = await program.account.loLv4Reply.all()

      var upVotedLoReply = loLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv4Reply.id))

      assert(upVotedLoReply[0].account.netVoteScore.eq(new anchor.BN(voteAmount)))

      //Down Vote Reply
      if(postDownVote)
      {
        await program.methods.loLv4ReplyVote
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoLv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
          usdcMint.publicKey,
          new anchor.BN(negativeVoteAmount)
        ).rpc()

        loLv4Replies = await program.account.loLv4Reply.all()

        var downVotedLoReply = loLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv4Reply.id))

        assert(downVotedLoReply[0].account.netVoteScore.eq(new anchor.BN(0)))
      }

      //Star Reply
      await program.methods.starLoLv4Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      loLv4Replies = await program.account.loLv4Reply.all()

      var starredLoReply = loLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv4Reply.id))

      assert(starredLoReply[0].account.isStarred == true)

      //Implement Idea
      await program.methods.setIdeaImplementedFlag
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        true
      ).rpc()

      var ideas = await program.account.idea.all()

      var implementedLoLv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(implementedLoLv4ReplyIdea[0].account.isImplemented == true)

      //Unimplement Idea
      await program.methods.setIdeaImplementedFlag
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        false
      ).rpc()

      ideas = await program.account.idea.all()

      var unimplementedLoLv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(unimplementedLoLv4ReplyIdea[0].account.isImplemented == false)

      //Update Idea
      await program.methods.updateIdea
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        updatedIdea
      ).rpc()

      ideas = await program.account.idea.all()

      var updatedLoLv4ReplyIdea = ideas.filter((idea: { account: { chatAccountPostCountIndex: anchor.BN, commentSectionNamePrefix: String, commentSectionName: String }}  ) =>
        ((idea.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1)))) &&
      ((idea.account.commentSectionNamePrefix == loCommentSectionNamePrefix) &&
      (idea.account.commentSectionName == commentSectionName))))

      assert(updatedLoLv4ReplyIdea[0].account.isUpdated == true)
      assert(updatedLoLv4ReplyIdea[0].account.idea == updatedIdea)

      //Unstar Reply
      if(unStar)
      {
        await program.methods.unstarLoLv4Reply
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoLv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        loLv4Replies = await program.account.loLv4Reply.all()

        var starredLoReply = loLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv4Reply.id))

        assert(starredLoReply[0].account.isStarred == false)
      }

      //FED Reply
      await program.methods.fedLoLv4Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        newLoLv4Reply.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
      ).rpc()

      loLv4Replies = await program.account.loLv4Reply.all()

      var fedLoReply = loLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv4Reply.id))

      assert(fedLoReply[0].account.isFed == true)

      //UnFED Reply
      if(unFED)
      {
        await program.methods.unfedLoLv4Reply
        (
          loCommentSectionNamePrefix, commentSectionName,
          newLoLv4Reply.postOwnerAddress,
          chatAccount.commentAndReplyCount.sub(new anchor.BN(1))
        ).rpc()

        loLv4Replies = await program.account.loLv4Reply.all()

        var fedLoReply = loLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv4Reply.id))

        assert(fedLoReply[0].account.isFed == false)
      }

      //Delete Reply
      await program.methods.deleteLoLv4Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey
      ).rpc()

      loLv4Replies = await program.account.loLv4Reply.all()

      var deletedLoReply = loLv4Replies.filter((reply: { account: { id: anchor.BN }}  ) => reply.account.id.eq(newLoLv4Reply.id))

      assert(deletedLoReply[0].account.isDeleted == true)

      //Reply To Reply
      const replyToLv4Reply = "Why you delete reply to reply to reply? :0"

      await program.methods.replyToLoLv4Reply
      (
        loCommentSectionNamePrefix, commentSectionName,
        deletedLoReply[0].account.postOwnerAddress,
        chatAccount.commentAndReplyCount.sub(new anchor.BN(1)),
        usdcMint.publicKey,
        replyToLv4Reply
      ).rpc()

      var chatAccount = await program.account.chatAccount.fetch(getChatAccountPDA(program.provider.publicKey))

      loLv4Replies = await program.account.loLv4Reply.all()

      var replyToLoLv4Reply = loLv4Replies.filter((reply: { account: { chatAccountPostCountIndex: anchor.BN }}  ) => reply.account.chatAccountPostCountIndex.eq(chatAccount.commentAndReplyCount.sub(new anchor.BN(1))))

      assert(replyToLoLv4Reply[0].account.msg == replyToLv4Reply)
    }

    /*while(true)
    {
      await sleepFunction()
    }*/
  })

  const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))
  var counter = 0
  async function sleepFunction() {
    console.log('Start sleep: ', counter)
    await sleep(5000) // Sleep for 5 seconds
    console.log('End sleep: ', counter)
    counter += 1
  }

  async function airDropSol(walletPublicKey: PublicKey)
  {
    let token_airdrop = await program.provider.connection.requestAirdrop(walletPublicKey, 
    100 * 1000000000) //1 billion lamports equals 1 SOL

    const latestBlockHash = await program.provider.connection.getLatestBlockhash()
    await program.provider.connection.confirmTransaction
    ({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: token_airdrop
    })
  }

  async function deriveWalletATA(walletPublicKey: PublicKey, tokenMintAddress: PublicKey)
  {
    return await Token.getAssociatedTokenAddress
    (
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      tokenMintAddress,
      walletPublicKey
    )
  }

  async function createATAForWallet(walletKeyPair: Keypair, tokenMintAddress: PublicKey, walletATA: PublicKey)
  {
    //1. Add createATA instruction to transaction
    const transaction = new Transaction().add
    (
      Token.createAssociatedTokenAccountInstruction
      (
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        tokenMintAddress,
        walletATA,
        walletKeyPair.publicKey,
        walletKeyPair.publicKey
      )
    )

    //2. Fetch the latest blockhash and set it on the transaction.
    const latestBlockhash = await program.provider.connection.getLatestBlockhash()
    transaction.recentBlockhash = latestBlockhash.blockhash
    transaction.feePayer = walletKeyPair.publicKey

    //3. Sign the transaction
    transaction.sign(walletKeyPair);
    //const signedTransaction = await program.provider.wallet.signTransaction(transaction)

    //4. Send the signed transaction to the network.
    //We get the signature back, which can be used to track the transaction.
    const tx = await program.provider.connection.sendRawTransaction(transaction.serialize())

    await program.provider.connection.confirmTransaction(tx, 'processed')
  }

  async function mintUSDCToWallet(tokenMintAddress: PublicKey, walletATA: PublicKey)
  {
    //1. Add createMintTo instruction to transaction
    const transaction = new Transaction().add
    (
      Token.createMintToInstruction
      (
        TOKEN_PROGRAM_ID,
        tokenMintAddress,
        walletATA,
        program.provider.publicKey,
        [testingWalletKeypair],
        10000000000//$10,000.00
      )
    )

    // 3. Send the transaction
    await program.provider.sendAndConfirm(transaction);
  }

  function getChatProtocolCEOAccountPDA()
  {
    const [chatProtocolCEOPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        new TextEncoder().encode("chatProtocolCEO")
      ],
      program.programId
    )
    return chatProtocolCEOPDA
  }

  function getPollPDA(pollIndex: number)
  {
    const [pollPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        utf8.encode("poll"),
        new anchor.BN(pollIndex).toBuffer('le', 16)
      ],
      program.programId
    )
    return pollPDA
  }

  function getPollOptionPDA(pollIndex: number, pollOptionIndex: number)
  {
    const [pollOptionPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        utf8.encode("pollOption"),
        new anchor.BN(pollIndex).toBuffer('le', 16),
        new anchor.BN(pollOptionIndex).toBuffer('le', 1)
      ],
      program.programId
    )
    return pollOptionPDA
  }

  function getChatAccountPDA(userAddress: anchor.web3.PublicKey)
  {
    const [chatAccountPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        utf8.encode("chatAccount"),
        userAddress.toBuffer()
      ],
      program.programId
    )
    return chatAccountPDA
  }
 
  function getCommentSectionPDA(commentSectionNamePrefix: string, commentSectionName: string)
  {
    const [commentSectionPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        utf8.encode("commentSection"),
        utf8.encode(commentSectionNamePrefix),
        utf8.encode(commentSectionName)
      ],
      program.programId
    )
    return commentSectionPDA
  }

  function getNewTime()
  {
    var newDate = new Date()
  
    return newDate.toLocaleTimeString('en-US', 
    { timeZone: 'America/New_York', 
      timeZoneName: "short"
    })
  }

  function getNewDate()
  {
    var newDate = new Date()
  
    return newDate.toLocaleDateString('en-US', 
    { 
      weekday: 'short',
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  }
})