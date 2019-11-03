module Main exposing (..)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode as JD
import Json.Decode.Pipeline
import Json.Encode as JE



-- MAIN


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }



-- MODEL


emptyProduct =
    { id = ""
    , name = ""
    , image = ""
    , price = 0
    }


type alias Product =
    { id : String
    , name : String
    , image : String
    , price : Int
    }


type alias Model =
    { products : List Product
    , currentProduct : Product
    , images : List String
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { products = []
      , currentProduct = emptyProduct
      , images = []
      }
    , Cmd.batch [ getProducts, getImages ]
    )



-- UPDATE


type Field
    = ProductName
    | ProductImage
    | ProductPrice


type Msg
    = GotProducts (Result Http.Error (List Product))
    | GotImages (Result Http.Error (List String))
    | GotNewProduct (Result Http.Error Product)
    | CreateProduct Product
    | Change Field String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CreateProduct product ->
            ( { model | currentProduct = emptyProduct }, postNewProduct <| product )

        GotProducts result ->
            case result of
                Ok products ->
                    ( { model | products = products }, Cmd.none )

                Err _ ->
                    ( model, Cmd.none )

        GotImages result ->
            case result of
                Ok images ->
                    ( { model | images = images }, Cmd.none )

                Err _ ->
                    ( model, Cmd.none )

        GotNewProduct result ->
            case result of
                Ok product ->
                    ( { model | products = List.append model.products [ product ] }, Cmd.none )

                Err _ ->
                    ( model, Cmd.none )

        Change field value ->
            case field of
                ProductName ->
                    let
                        old =
                            model.currentProduct

                        newProduct =
                            { old | name = value }
                    in
                    ( { model | currentProduct = newProduct }, Cmd.none )

                ProductImage ->
                    let
                        old =
                            model.currentProduct
                    in
                    ( { model | currentProduct = { old | image = value } }, Cmd.none )

                ProductPrice ->
                    let
                        old =
                            model.currentProduct

                        newProduct =
                            { old
                                | price =
                                    case String.toInt value of
                                        Just number ->
                                            number

                                        Nothing ->
                                            0
                            }
                    in
                    ( { model | currentProduct = newProduct }, Cmd.none )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none



-- VIEW


view : Model -> Html Msg
view model =
    div []
        [ h2 [] [ text "Awesome products" ]
        , viewProducts model
        , createProductView model.currentProduct model.images
        ]


viewProducts : Model -> Html Msg
viewProducts model =
    div []
        [ ul [] (model.products |> List.map viewProduct)
        ]


createProductView : Product -> List String -> Html Msg
createProductView currentProduct images =
    div []
        [ div []
            [ text "Name"
            , input [ type_ "text", value currentProduct.name, onInput (Change ProductName) ] []
            ]
        , div []
            [ text "Image"
            , select [ onInput (Change ProductImage) ] (List.append [ option [] [ text "None" ] ] (List.map (viewImage currentProduct.image) images))
            ]
        , div []
            [ text "Price (in Cent)"
            , input [ type_ "number", value (String.fromInt currentProduct.price), onInput (Change ProductPrice) ] []
            ]
        , div [] [ button [ onClick (CreateProduct currentProduct) ] [ text "Create" ] ]
        ]


viewImage : String -> String -> Html Msg
viewImage currentImage image =
    option [ value image, selected (currentImage == image) ] [ text image ]


viewProduct : Product -> Html msg
viewProduct product =
    li [] [ text (product.name ++ " costs " ++ String.fromInt product.price ++ " image: " ++ product.image) ]


getProducts : Cmd Msg
getProducts =
    Http.get
        { url = "/products"
        , expect = Http.expectJson GotProducts productsDecoder
        }


getImages : Cmd Msg
getImages =
    Http.get
        { url = "/images"
        , expect = Http.expectJson GotImages imagesDecoder
        }


postNewProduct : Product -> Cmd Msg
postNewProduct product =
    Http.post
        { url = "/products"
        , body = Http.jsonBody <| productEncoder product
        , expect = Http.expectJson GotNewProduct productDecoder
        }


imagesDecoder : JD.Decoder (List String)
imagesDecoder =
    JD.list JD.string


productsDecoder : JD.Decoder (List Product)
productsDecoder =
    JD.list productDecoder


productDecoder : JD.Decoder Product
productDecoder =
    JD.succeed Product
        |> Json.Decode.Pipeline.required "id" JD.string
        |> Json.Decode.Pipeline.required "name" JD.string
        |> Json.Decode.Pipeline.required "image" JD.string
        |> Json.Decode.Pipeline.required "price" JD.int


productEncoder : Product -> JE.Value
productEncoder product =
    JE.object
        [ ( "id", JE.string product.id )
        , ( "name", JE.string product.name )
        , ( "price", JE.int product.price )
        , ( "image", JE.string product.image )
        ]
