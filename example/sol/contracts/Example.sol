// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

// Uncomment this line to use console.log
// import "hardhat/console.sol";

contract Example {
    address payable public owner;

    event Withdrawal(uint amount, uint when);

    struct Data {
        uint256 from;
        uint256 to;
    }

    constructor() payable {
        owner = payable(msg.sender);
    }

    function withdraw() public {
        // Uncomment this line, and the import of "hardhat/console.sol", to print a log in your terminal
        // console.log("Unlock time is %o and block timestamp is %o", unlockTime, block.timestamp);

        require(msg.sender == owner, "You aren't the owner");

        emit Withdrawal(address(this).balance, block.timestamp);

        owner.transfer(address(this).balance);
    }

    function get_from(Data memory data) public pure returns (uint256) {
        return data.from;
    }

    function get_to(Data memory data) public pure returns (uint256) {
        return data.to;
    }

    function get() public pure returns (Data memory) {
        return Data(1000, 140);
    }
}
