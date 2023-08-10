// SPDX-License-Identifier: MIT
pragma solidity =0.8.19;

import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

contract AccessGrants {
    struct Grant {
      address owner;
      address grantee;
      string dataId;
      uint256 lockedUntil;
    }
    
    mapping(bytes32 => Grant) private grantsById;
    using EnumerableSet for EnumerableSet.Bytes32Set;
    mapping(address => EnumerableSet.Bytes32Set) private grantIdsByOwner;
    mapping(address => EnumerableSet.Bytes32Set) private grantIdsByGrantee;
    mapping(string => EnumerableSet.Bytes32Set) private grantIdsByDataId;

    constructor() {}

    function insert_grant(address _grantee, string memory _dataId) public {
        Grant memory newGrant = Grant({
            owner: msg.sender,
            grantee: _grantee,
            dataId: _dataId,
            lockedUntil: 0
        });
        
        bytes32 newGrantId = keccak256(abi.encodePacked(newGrant.owner, newGrant.grantee, newGrant.dataId, newGrant.lockedUntil));

        grantsById[newGrantId] = newGrant;
        grantIdsByOwner[newGrant.owner].add(newGrantId);
        grantIdsByGrantee[newGrant.grantee].add(newGrantId);
        grantIdsByDataId[newGrant.dataId].add(newGrantId);
    }

    function delete_grant(address _grantee, string memory _dataId) public returns (Grant memory) {
        Grant memory grant = grants_by(msg.sender, _grantee, _dataId)[0];

        bytes32 grantId = keccak256(abi.encodePacked(grant.owner, grant.grantee, grant.dataId, grant.lockedUntil));

        delete grantsById[grantId];
        grantIdsByOwner[grant.owner].remove(grantId);
        grantIdsByGrantee[grant.grantee].remove(grantId);
        grantIdsByDataId[grant.dataId].remove(grantId);

        return grant;
    }

    function grants_for(address _grantee, string memory _dataId) public view returns (Grant[] memory) {
        return grants_by(address(0), _grantee, _dataId);
    }

    function grants_by(address owner, address grantee, string memory dataId) public view returns (Grant[] memory) {
        bytes32[] memory candidateGrantIds;
        uint256 countCandidateGrants;

        if (owner != address(0)) {
          candidateGrantIds = grantIdsByOwner[owner].values();
          countCandidateGrants = grantIdsByOwner[owner].length();
        } else {
          candidateGrantIds = grantIdsByGrantee[grantee].values();
          countCandidateGrants = grantIdsByGrantee[grantee].length();
        }

        uint256 returnCount = 0;
        bool[] memory keepList = new bool[](countCandidateGrants);

        for (uint i = 0; i < countCandidateGrants; i++) {
            bytes32 grantId = candidateGrantIds[i];
            bool keep = true;

            if (grantee != address(0)) {
                keep = grantIdsByGrantee[grantee].contains(grantId);
            }

            if (keep && keccak256(abi.encodePacked((dataId))) != keccak256(abi.encodePacked(("0")))) {
                keep = grantIdsByDataId[dataId].contains(grantId);
            }

            if (keep) {
                returnCount++;
                keepList[i] = keep;
            }
        }

        Grant[] memory _grants = new Grant[](returnCount);

        uint256 returnIndex = 0;
        for (uint256 i = 0; i < countCandidateGrants; i++) {
            if (keepList[i]) {
                _grants[returnIndex] = grantsById[candidateGrantIds[i]];
                returnIndex++;
            }
        }

        return _grants;
    }
}
