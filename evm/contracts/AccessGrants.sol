// SPDX-License-Identifier: MIT
pragma solidity =0.8.19;

import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

contract AccessGrants {
    using EnumerableSet for EnumerableSet.Bytes32Set;

    struct Grant {
      address owner;
      address grantee;
      string dataId;
      uint256 lockedUntil;
    }

    mapping(bytes32 => Grant) private grantsById;
    mapping(address => EnumerableSet.Bytes32Set) private grantIdsByOwner;
    mapping(address => EnumerableSet.Bytes32Set) private grantIdsByGrantee;
    mapping(string => EnumerableSet.Bytes32Set) private grantIdsByDataId;

    bytes32 constant WILDCARD_DATA_ID = keccak256(abi.encodePacked("0"));

    constructor() {}

    /*
     * public interface
     */

    function insert_grant(
      address grantee,
      string memory dataId
      // TODO uint256 lockedUntil
    ) public {
        Grant memory grant = Grant({
            owner: msg.sender,
            grantee: grantee,
            dataId: dataId,
            // TODO lockedUntil: lockedUntil
            lockedUntil: 0
        });

        bytes32 grantId = deriveGrantId(grant);

        require(grantsById[grantId].owner == address(0), "Grant already exists");

        grantsById[grantId] = grant;
        grantIdsByOwner[grant.owner].add(grantId);
        grantIdsByGrantee[grant.grantee].add(grantId);
        grantIdsByDataId[grant.dataId].add(grantId);
    }

    function delete_grant(
      address grantee,
      string memory dataId
    ) public returns (Grant memory) {
        Grant[] memory grants = grants_by(msg.sender, grantee, dataId);

        require(grants.length > 0, "No grants for sender");

        Grant memory grant = grants[0];

        bytes32 grantId = deriveGrantId(grant);

        delete grantsById[grantId];
        grantIdsByOwner[grant.owner].remove(grantId);
        grantIdsByGrantee[grant.grantee].remove(grantId);
        grantIdsByDataId[grant.dataId].remove(grantId);

        return grant;
    }

    function grants_for(
      address grantee,
      string memory dataId
    ) public view returns (Grant[] memory) {
        return grants_by(address(0), grantee, dataId);
    }

    function grants_by(
      address owner,
      address grantee,
      string memory dataId
    ) public view returns (Grant[] memory) {
        bytes32[] memory candidateGrantIds;
        uint256 candidateGrantCount;

        if (owner != address(0)) {
          candidateGrantIds = grantIdsByOwner[owner].values();
          candidateGrantCount = grantIdsByOwner[owner].length();
        } else if (grantee != address(0)) {
          candidateGrantIds = grantIdsByGrantee[grantee].values();
          candidateGrantCount = grantIdsByGrantee[grantee].length();
        } else {
          revert("Neither owner nor grantee provided");
        }

        uint256 returnCount = 0;
        bool[] memory keepList = new bool[](candidateGrantCount);

        for (uint256 i = 0; i < candidateGrantCount; i++) {
            bytes32 candidateGrantId = candidateGrantIds[i];
            bool returnCandidate = false;

            returnCandidate =
              grantee == address(0)
              || grantIdsByGrantee[grantee].contains(candidateGrantId);

            returnCandidate = returnCandidate && (
              isWildcardDataId(dataId)
              || grantIdsByDataId[dataId].contains(candidateGrantId)
            );

            if (returnCandidate) {
                returnCount++;
                keepList[i] = true;
            }
        }

        Grant[] memory grants = new Grant[](returnCount);
        uint256 returnIndex = 0;

        for (uint256 i = 0; i < candidateGrantCount; i++) {
            if (keepList[i]) {
                grants[returnIndex] = grantsById[candidateGrantIds[i]];
                returnIndex++;
            }
        }

        return grants;
    }

    /*
     * private helpers
     */

    function deriveGrantId(
      Grant memory grant
    ) private pure returns (bytes32) {
        return keccak256(abi.encodePacked(
          grant.owner,
          grant.grantee,
          grant.dataId,
          grant.lockedUntil
        ));
    }

    function isWildcardDataId(
        string memory dataId
    ) private pure returns (bool) {
      return keccak256(abi.encodePacked((dataId))) == WILDCARD_DATA_ID;
    }
}
