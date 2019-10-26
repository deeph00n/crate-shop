module Main exposing (..)
import Browser
import Html exposing (..)
import Http
import Json.Decode exposing (Decoder, field, string)
import Json.Decode as Decode exposing (Decoder, int, string, float)
import Json.Decode.Pipeline exposing (required, optional, hardcoded)


-- MAIN


main =
  Browser.element
    { init = init
    , update = update
    , subscriptions = subscriptions
    , view = view
    }



-- MODEL


type Model
  = Failure
  | Loading
  | Success (List Product)


init : () -> (Model, Cmd Msg)
init _ =
  (Loading, getProducts)



-- UPDATE


type Msg
  = GotProducts (Result Http.Error (List Product))


update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    GotProducts result ->
      case result of
        Ok products ->
          (Success products, Cmd.none)

        Err _ ->
          (Failure, Cmd.none)



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
  Sub.none



-- VIEW


view : Model -> Html Msg
view model =
  div []
    [ h2 [] [ text "Awesome products" ]
    , viewProducts model
    ]


viewProducts : Model -> Html Msg
viewProducts model =
  case model of
    Failure ->
      div [] [ text "I could not load products for some reason. " ]

    Loading ->
      text "Loading..."

    Success products ->
      ul [] (List.map viewProduct (List.sortBy (\p -> p.name) products))
-- HTTP

viewProduct : Product -> Html msg
viewProduct product =
    li [] [ text (product.name ++ " costs " ++ (String.fromInt product.price)) ]

getProducts : Cmd Msg
getProducts =
  Http.get
    { url = "/product"
    , expect = Http.expectJson GotProducts productsDecoder
    }

type alias Product =
  { price : Int
  , name : String
  }

productsDecoder : Decoder (List Product)
productsDecoder =
    Decode.list (Decode.succeed Product
                     |> required "price" int
                     |> required "name" string)
