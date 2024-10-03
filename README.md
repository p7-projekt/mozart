# Docker

To build image:
```
docker build -t ghc-instance .
```

To run the container:
```
docker run -p 8080:8080 -d ghc-instance
```

# Example Json
Following JSON will compile correctly and return passed test cases.
This is only temporary and will return more complete response in the future.

```json
{
  "solution": "solution x =\n  if x < 0\n    then x * (-1)\n    else x",
  "testCases": [
    {
      "id": 0,
      "inputParameters": [
        {
          "valueType": "integer",
          "value": "-5"
        }
      ],
      "outputParameters": [
        {
          "valueType": "integer",
          "value": "5"
        }
      ]
    },
    {
      "id": 1,
      "inputParameters": [
        {
          "valueType": "integer",
          "value": "5"
        }
      ],
      "outputParameters": [
        {
          "valueType": "integer",
          "value": "5"
        }
      ]
    }
  ]
}
```
