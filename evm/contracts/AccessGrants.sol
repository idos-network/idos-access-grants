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

    mapping(bytes32 => Grant) private _grantsById;

    mapping(address => EnumerableSet.Bytes32Set) private _grantIdsByOwner;
    mapping(address => EnumerableSet.Bytes32Set) private _grantIdsByGrantee;
    mapping(string => EnumerableSet.Bytes32Set) private _grantIdsByDataId;

    // CURIOSITY Why not the empty string?
    bytes32 private constant _WILDCARD_DATA_ID = keccak256(abi.encodePacked("0"));

    constructor() {}

    function insertGrant(
        address grantee,
        string memory dataId,
        uint256 lockedUntil
    ) external {
        Grant memory grant = Grant({
            owner: msg.sender,
            grantee: grantee,
            dataId: dataId,
            lockedUntil: lockedUntil
        });

        bytes32 grantId = _deriveGrantId(grant);

        require(_grantsById[grantId].owner == address(0), "Grant already exists");

        _grantsById[grantId] = grant;
        _grantIdsByOwner[grant.owner].add(grantId);
        _grantIdsByGrantee[grant.grantee].add(grantId);
        _grantIdsByDataId[grant.dataId].add(grantId);
    }

    function deleteGrant(
        address grantee,
        string memory dataId,
        uint256 lockedUntil
    ) external {
        Grant[] memory grants = findGrants(msg.sender, grantee, dataId);

        require(grants.length > 0, "No grants for sender");

        for (uint256 i = 0; i < grants.length; i++) {
            Grant memory grant = grants[i];

            if (lockedUntil == 0 || grants[i].lockedUntil == lockedUntil) {
                // REVIEWME, low prio. Please remember that `block.timestamp` can be
                // manipulated by an incentivized miner. I don't think that's a big
                // deal here, since the practical drift in less than a day, AFAIK.
                require(grant.lockedUntil < block.timestamp, "Grant is timelocked");

                bytes32 grantId = _deriveGrantId(grant);

                delete _grantsById[grantId];
                _grantIdsByOwner[grant.owner].remove(grantId);
                _grantIdsByGrantee[grant.grantee].remove(grantId);
                _grantIdsByDataId[grant.dataId].remove(grantId);
            }
        }
    }

    function grantsFor(
        address grantee,
        string memory dataId
    ) external view returns (Grant[] memory) {
        return findGrants(address(0), grantee, dataId);
    }

    function findGrants(
        address owner,
        address grantee,
        string memory dataId
    ) public view returns (Grant[] memory) {
        bytes32[] memory candidateGrantIds;
        uint256 candidateGrantCount;

        if (owner != address(0)) {
            candidateGrantIds = _grantIdsByOwner[owner].values();
            candidateGrantCount = _grantIdsByOwner[owner].length();
        } else if (grantee != address(0)) {
            candidateGrantIds = _grantIdsByGrantee[grantee].values();
            candidateGrantCount = _grantIdsByGrantee[grantee].length();
        } else {
            // REVIEWME This is not the same as other implementations. Namely, this one doesn't allow for
            // (None, None, Some(data_id)). Is that on purpose, or accidental design?
            revert("You must provide some search criteria");
        }

        uint256 returnCount = 0;
        bool[] memory keepList = new bool[](candidateGrantCount);

        for (uint256 i = 0; i < candidateGrantCount; i++) {
            bytes32 candidateGrantId = candidateGrantIds[i];

            bool returnCandidate =
                (
                    grantee == address(0) || _grantIdsByGrantee[grantee].contains(candidateGrantId)
                ) && (
                    _isWildcardDataId(dataId) || _grantIdsByDataId[dataId].contains(candidateGrantId)
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
                grants[returnIndex] = _grantsById[candidateGrantIds[i]];
                returnIndex++;
            }
        }

        return grants;
    }

    function _deriveGrantId(
        Grant memory grant
    ) private pure returns (bytes32) {
        return keccak256(abi.encodePacked(
            grant.owner,
            grant.grantee,
            grant.dataId,
            grant.lockedUntil
        ));
    }

    function _isWildcardDataId(
        string memory dataId
    ) private pure returns (bool) {
        return keccak256(abi.encodePacked((dataId))) == _WILDCARD_DATA_ID;
    }
}
