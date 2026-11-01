# example — remove when building your service (notes demo slice)
#
# The notes contract, written as the behaviour a product owner would approve.
# Steps live in tests/acceptance/steps/; the runner is tests/acceptance/runner.rs.

Feature: Notes
  A note can be created through the API and read back.

  Background:
    Given a unique run label

  Scenario: A new note round-trips through the API
    When I create a note with owner "acceptance" and text "buy milk"
    Then the response status is 201
    And the response is the created note
    When I read the notes list
    Then the note I created appears in the list

  Scenario: A note longer than the limit is rejected
    When I create a note with owner "acceptance" and a text of 301 characters
    Then the response status is 400
    And the response is a problem document
    And the problem points at "/note"
