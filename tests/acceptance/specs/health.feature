Feature: Health
  The service reports its dependencies through readiness.

  Scenario: Readiness is green when the dependencies are reachable
    When I request "/readyz"
    Then the response status is 204
