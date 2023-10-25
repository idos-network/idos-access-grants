# NEAR.ts


Initialized with `npx create-near-app` ([docs](https://docs.near.org/sdk/near-sdk-js/reference))

---

```
yarn test
```

If you happen to have trouble running it locally, there's also a docker image that runs the tests as part of the build process. Run:
```bash
docker build . -t near-ts && docker run -it --rm near-ts yarn test
```

This is especially useful if you're running an arm64 (like an M1 or M2).
